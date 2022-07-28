use crate::ThreadPool;

pub struct RayonThreadPool {
    inner: rayon::ThreadPool,
}

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            inner: rayon::ThreadPoolBuilder::new()
                .num_threads(threads as usize)
                .build()
                .expect("fail to create rayon tp"),
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + 'static + Send,
    {
        self.inner.spawn(job);
    }
}
