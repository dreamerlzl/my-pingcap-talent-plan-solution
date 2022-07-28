use crate::Result;

pub trait ThreadPool: Send {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + 'static + Send;
}

mod naive;
pub use naive::NaiveThreadPool;

mod shared;
pub use shared::SharedQueueThreadPool;

mod rayon;
pub use self::rayon::RayonThreadPool;
