pub type BoxFuture<'a, T> = futures::future::BoxFuture<'a, T>;

#[derive(Debug)]
pub struct TaskHandle(tokio::task::JoinHandle<()>);

impl TaskHandle {
    #[inline]
    pub fn abort(&self) {
        self.0.abort();
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }
}

#[inline]
pub fn spawn(task: impl futures::Future<Output = ()> + Send + 'static) -> TaskHandle {
    TaskHandle(tokio::spawn(task))
}

pub trait Boxable<'a>: futures::Future + Sized + Send + 'a {
    fn boxed(self) -> BoxFuture<'a, Self::Output> {
        futures::FutureExt::boxed(self)
    }
}
impl<'a, F: futures::Future + Sized + Send + 'a> Boxable<'a> for F {}
