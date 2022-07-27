use std::{sync::Arc, thread};

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::ThreadPool;

type Job = Box<dyn Send + 'static + FnOnce()>;
enum Message {
    Task(Job),
    Stop,
}

pub struct SharedQueueThreadPool {
    msg_send_queue: Sender<Message>,
    threads: u32,
}

pub struct ThreadPoolSharedData {
    msg_queue: Receiver<Message>,
}

impl ThreadPoolSharedData {
    fn new(msg_queue: Receiver<Message>) -> Self {
        Self { msg_queue }
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.threads {
            self.msg_send_queue
                .send(Message::Stop)
                .expect("fail to send stop signal to tp");
        }
    }
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let (tx, rx) = unbounded();
        let shared_data = Arc::new(ThreadPoolSharedData::new(rx));
        for _ in 0..threads {
            spawn_thread(shared_data.clone());
        }
        Ok(Self {
            msg_send_queue: tx,
            threads,
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + 'static + Send,
    {
        self.msg_send_queue
            .send(Message::Task(Box::new(job)))
            .expect("fail to send job");
    }
}

struct Sentinel {
    shared_data: Arc<ThreadPoolSharedData>,
    active: bool,
}

impl Sentinel {
    fn new(shared_data: Arc<ThreadPoolSharedData>) -> Self {
        Self {
            shared_data,
            active: true,
        }
    }

    fn cancel(&mut self) {
        self.active = false;
    }
}

impl Drop for Sentinel {
    fn drop(&mut self) {
        if self.active && thread::panicking() {
            spawn_thread(self.shared_data.clone());
        }
    }
}

// spawn one thread
// used in either threadpool creation
// or after a thread paniced
fn spawn_thread(shared_data: Arc<ThreadPoolSharedData>) {
    thread::spawn(move || {
        let mut sentinel = Sentinel::new(shared_data.clone());
        loop {
            let msg = shared_data.msg_queue.recv().expect("receiver closed");
            match msg {
                Message::Task(job) => job(),
                Message::Stop => break,
            }
        }
        sentinel.cancel();
    });
}
