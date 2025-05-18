pub type BoxFuture<'a, T> = futures::future::BoxFuture<'a, T>;

pub trait Boxable<'a>: futures::Future + Sized + Send + 'a {
    fn boxed(self) -> BoxFuture<'a, Self::Output> {
        futures::FutureExt::boxed(self)
    }
}
impl<'a, F: futures::Future + Sized + Send + 'a> Boxable<'a> for F {}
