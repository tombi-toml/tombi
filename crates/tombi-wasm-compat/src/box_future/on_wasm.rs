pub type BoxFuture<'a, T> = futures::future::LocalBoxFuture<'a, T>;

pub trait Boxable<'a>: futures::Future + Sized + 'a {
    fn boxed(self) -> BoxFuture<'a, Self::Output> {
        futures::FutureExt::boxed_local(self)
    }
}
impl<'a, F: futures::Future + Sized + 'a> Boxable<'a> for F {}
