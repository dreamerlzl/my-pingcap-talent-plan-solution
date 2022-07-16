use crate::ThreadPool;

pub struct RayonThreadPool {}

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> crate::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + 'static + Send,
    {
        todo!()
    }
}
