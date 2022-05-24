use std::{
    error::Error,
    fmt,
    io::{Read, Write},
    net::TcpStream,
};

use mbytes::extract::{extract_bulk_string, extract_simple_string, resp_to_vec};
use mbytes::{get_to_bytes, ping_to_bytes};
use mserde::de::from_bytes;
use mserde::RESP;

#[derive(Debug, Clone)]
enum MyError {
    Fail(Option<String>),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fuck")
    }
}

impl Error for MyError {}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct MyCli {
    stream: TcpStream,
}

impl MyCli {
    pub fn new(host: String, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        println!("connecting to {:?}", addr);
        let stream = TcpStream::connect(addr)?;
        println!("connection success");
        Ok(MyCli { stream })
    }

    pub fn get(&mut self, k: &str) -> Result<Option<Vec<u8>>> {
        let bytes = mserde::get_to_bytes(k)?;
        self.stream.write_all(bytes.as_slice())?;
        let mut buf = [0; 128];
        let num_bytes = self.stream.read(&mut buf)?;
        match from_bytes::<RESP>(&buf[..num_bytes])? {
            RESP::BulkString(v) => Ok(v),
            s => panic!("unexpected response type {:?}", s),
        }
    }

    pub fn set<T: ToString>(&mut self, k: String, v: T) -> Result<()> {
        let bytes = mserde::set_to_bytes(k, v.to_string())?;
        self.stream.write_all(bytes.as_slice())?;
        let mut buf = [0; 128];
        let num_bytes = self.stream.read(&mut buf)?;
        match from_bytes::<RESP>(&buf[..num_bytes])? {
            RESP::SimpleString(s) => {
                if s == "OK" {
                    Ok(())
                } else {
                    panic!("unexpected ok string: {}", s)
                }
            }
            RESP::BulkString(None) => {
                Err(Box::new(MyError::Fail(Some("not performed!".to_owned()))))
            }
            ch => {
                panic!("unexpected RESP Type {:?}", ch)
            }
        }
    }

    pub fn ping(&mut self) -> Result<bool> {
        let bytes = ping_to_bytes(None)?;
        self.stream.write_all(bytes.as_slice())?;
        let mut buf = [0; 7];
        let num_bytes = self.stream.read(&mut buf)?;
        let (pong, _) = extract_simple_string(&buf[..num_bytes])?;
        println!("{pong}");
        Ok(true)
    }

    pub fn ping2(&mut self) -> Result<bool> {
        let bytes = mserde::ping_to_bytes(None)?;
        self.stream.write_all(bytes.as_slice())?;
        let mut buf = [0; 7];
        let num_bytes = self.stream.read(&mut buf)?;
        let (pong, _) = extract_simple_string(&buf[..num_bytes])?;
        println!("{pong}");
        Ok(true)
    }

    pub fn ping_str(&mut self, s: String) -> Result<bool> {
        let bytes = ping_to_bytes(Some(s))?;
        self.stream.write_all(bytes.as_slice())?;
        let mut buf = [0; 512];
        let num_bytes = self.stream.read(&mut buf)?;
        // let (pong, _) = extract_simple_string(&buf[..num_bytes])?;
        //let result = resp_to_vec(&buf[..num_bytes])?
        //    .into_iter()
        //    .flat_map(|bs| bs.map(String::from_utf8))
        //    .collect::<Vec<std::result::Result<String, _>>>()
        //    .into_iter()
        //    .collect::<std::result::Result<Vec<String>, _>>()?;
        //let result = result.join(" ");

        let (result, _) = extract_bulk_string(&buf[..num_bytes])?;
        if let Some(result) = result.map(String::from_utf8) {
            let pong = result?;
            println!("{pong}");
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
