use std::thread;

use super::ThreadPool;

pub struct NaiveThreadPool {}

impl ThreadPool for NaiveThreadPool {
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + 'static + Send,
    {
        thread::spawn(job);
    }

    fn new(threads: u32) -> crate::Result<Self>
    where
        Self: Sized,
    {
        assert!(threads > 0);
        Ok(NaiveThreadPool {})
    }
}
