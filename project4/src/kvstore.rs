use std::collections::HashMap;
use std::fs::{rename, File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, SeekFrom};
use std::path::{Path, PathBuf};

use bson::Document;
use serde::{Deserialize, Serialize};

use crate::kvserror::{KvsError, Result};
use crate::KvsEngine;

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Set(String, String),
    Rm(String),
}

type FileID = u32;

pub struct KvStore {
    buf_writer: BufWriter<File>,
    log_index: HashMap<String, ValuePos>,
    log_dir_path: PathBuf,
    first_file_id: FileID,
    active_file_id: FileID,
}

#[derive(Clone)]
struct ValuePos {
    offset: u64,
    file_id: FileID,
}

impl ValuePos {
    fn new(file_id: FileID, offset: u64) -> Self {
        ValuePos { offset, file_id }
    }
}

const BACKUP_SUFFIX: &str = "bak";
const CHUNK_SIZE_BYTES: u64 = 1024 * 1024; // 4KB

// const CHUNK_SIZE_BYTES: u64 = 32; // for testing

impl KvStore {
    pub fn open(log_dir_path: impl AsRef<Path>) -> Result<Self> {
        // build the index
        let (log_index, first_file_id, active_file_id) = build_index(log_dir_path.as_ref())?;
        // dbg!((get_current_pos(&mut buf_writer)?, &log_pointer));

        // a bufwriter for the current active log
        let active_log_path = path_from_id(&PathBuf::from(log_dir_path.as_ref()), active_file_id);
        let active_log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(active_log_path)?;
        let buf_writer = BufWriter::new(active_log_file);

        let log_dir_path = PathBuf::from(log_dir_path.as_ref().clone());
        let kvstore = KvStore {
            buf_writer,
            log_index,
            log_dir_path,
            first_file_id,
            active_file_id,
        };
        Ok(kvstore)
    }

    fn read_value(&self, pos: &ValuePos) -> Result<String> {
        let log_file = OpenOptions::new()
            .read(true)
            .open(path_from_id(&self.log_dir_path, pos.file_id))?;

        let mut buf_reader = BufReader::new(log_file);
        buf_reader.seek(SeekFrom::Current(pos.offset as i64))?;
        let command = bson::from_reader(&mut buf_reader)?;
        match command {
            Command::Set(_, value) => Ok(value),
            Command::Rm(_) => panic!("the value position should never be rm"),
        }
    }

    // append a command
    fn append_command(&mut self, command: Command) -> Result<()> {
        let bytes = command_to_bytes(&command)?;
        self.buf_writer.write_all(bytes.as_slice())?;
        let current_size = get_current_pos(&mut self.buf_writer)?;
        match command {
            Command::Set(k, _) => {
                let offset = current_size - bytes.len() as u64;
                self.log_index
                    .insert(k, ValuePos::new(self.active_file_id, offset));
            }
            _ => {}
        }
        if current_size > CHUNK_SIZE_BYTES as u64 {
            self.buf_writer.flush()?;
            // this will compact logs, rebuild the index, and update the active file id
            self.active_file_id = self.compact_logs(self.active_file_id)?;
            let active_log_path =
                path_from_id(&PathBuf::from(&self.log_dir_path), self.active_file_id);
            let active_log_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(active_log_path)?;
            self.buf_writer = BufWriter::new(active_log_file);
        }
        Ok(())
    }

    // compacts all logs with id <= last_file_id
    // rebuild the log index
    // and returns the file id of the last compacted log (in time order)
    fn compact_logs(&mut self, last_file_id: FileID) -> Result<FileID> {
        let mut kv_snapshot = HashMap::new();
        for i in self.first_file_id..=last_file_id {
            let log_path = path_from_id(&self.log_dir_path, i);
            let file = OpenOptions::new()
                .read(true)
                .open(&log_path)
                .expect(&format!(
                    "fail to open log {}",
                    log_path.as_os_str().to_str().unwrap()
                ));

            let mut buf_reader = BufReader::new(file);
            while let Ok(doc) = Document::from_reader(&mut buf_reader) {
                let command = bson::from_document(doc)?;
                match command {
                    Command::Set(k, v) => kv_snapshot.insert(k, v),
                    Command::Rm(k) => kv_snapshot.remove(&k),
                };
            }
        }

        let mut current_log_bytes = 0;
        let mut current_file_id = self.first_file_id;
        let mut file_path = compact_path_from_id(&self.log_dir_path, current_file_id);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        let mut buf_writer = BufWriter::new(file);
        for (k, v) in kv_snapshot.into_iter() {
            let command = Command::Set(k, v);
            let bytes = command_to_bytes(&command)?;
            buf_writer
                .write_all(bytes.as_slice())
                .expect("fail to append in compacted log");
            current_log_bytes += bytes.len() as u64;
            if current_log_bytes > CHUNK_SIZE_BYTES {
                buf_writer.flush()?;
                rename(file_path, path_from_id(&self.log_dir_path, current_file_id))
                    .expect("fail to rename bak to log");
                current_log_bytes = 0;
                current_file_id += 1;
                file_path = compact_path_from_id(&self.log_dir_path, current_file_id);
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)?;
                buf_writer = BufWriter::new(file);
            }
        }
        rename(file_path, path_from_id(&self.log_dir_path, current_file_id))
            .expect("fail to rename bak to log");
        Ok(current_file_id)
    }
}

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = Command::Set(key.clone(), value);
        self.append_command(command)?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        self.buf_writer.flush()?;
        if let Some(pos) = self.log_index.get(&key) {
            // dbg!((key, pos));
            Ok(Some(self.read_value(pos)?))
        } else {
            Ok(None)
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if let Some(_) = self.log_index.remove(&key) {
            let command = Command::Rm(key);
            self.append_command(command)?;
            Ok(())
        } else {
            Err(KvsError::KeyNotFound(key))
        }
    }
}

fn command_to_bytes(command: &Command) -> Result<Vec<u8>> {
    let command_bson = bson::to_bson(command)?;
    let command_bytes = bson::to_vec(&command_bson)?;
    Ok(command_bytes)
}

fn get_current_pos<T: Seek>(file: &mut T) -> Result<u64> {
    let pos = file.seek(std::io::SeekFrom::Current(0))?;
    Ok(pos)
}

// return (file id, log paths) in time order
fn older_log_paths(log_dir_path: &Path) -> Vec<(FileID, PathBuf)> {
    let mut paths: Vec<_> = std::fs::read_dir(log_dir_path)
        .expect("fail to read data dir")
        .map(|r| r.expect(&"fail to read data file"))
        .collect();

    paths.sort_by_key(|de| de.path());
    let first_log_path = paths.get(0);
    if first_log_path.is_none() {
        return vec![];
    }
    let first_log_path = first_log_path.unwrap().path();

    let path_str = first_log_path
        .file_stem()
        .expect("fail to get filename of first log")
        .to_str()
        .expect("fail to find first log");
    let first_file_id: FileID = path_str.parse().expect("fail to parse the first file id");

    paths
        .into_iter()
        .enumerate()
        .map(|(i, de)| (i as FileID + first_file_id, de.path()))
        .collect()
}

// build the log index based on the existing logs
// and return the first and the last file id
fn build_index(log_dir_path: &Path) -> Result<(HashMap<String, ValuePos>, FileID, FileID)> {
    let mut log_pointer = HashMap::new();
    let mut log_paths = older_log_paths(log_dir_path);
    for (file_id, log_path) in log_paths.iter() {
        let file_id = *file_id;
        let file = OpenOptions::new()
            .read(true)
            .open(&log_path)
            .expect("fail to open log");
        let mut buf_reader = BufReader::new(file);
        let mut offset = 0;
        while let Ok(doc) = Document::from_reader(&mut buf_reader) {
            let command = bson::from_document(doc)?;
            match command {
                Command::Set(k, _) => log_pointer.insert(k, ValuePos { offset, file_id }),
                Command::Rm(k) => log_pointer.remove(&k),
            };
            offset = get_current_pos(&mut buf_reader)?;
        }
    }
    let first_file_id = if let Some((first_file_id, _)) = log_paths.get(0) {
        *first_file_id
    } else {
        0
    };
    if let Some((active_file_id, _)) = log_paths.pop() {
        Ok((log_pointer, first_file_id, active_file_id))
    } else {
        Ok((log_pointer, first_file_id, 0))
    }
}

fn path_from_id(log_dir_path: &PathBuf, file_id: FileID) -> PathBuf {
    log_dir_path.join(format!("{}.log", file_id))
}

fn compact_path_from_id(log_dir_path: &PathBuf, file_id: FileID) -> PathBuf {
    let mut file_path = path_from_id(log_dir_path, file_id);
    file_path.set_extension(BACKUP_SUFFIX);
    file_path
}
