#[cfg(not(feature = "wasm-send"))]
pub type BoxFuture<'a, T> = futures::future::LocalBoxFuture<'a, T>;

#[cfg(feature = "wasm-send")]
pub type BoxFuture<'a, T> = futures::future::BoxFuture<'a, T>;

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct TaskHandle {
    abort_handle: futures::future::AbortHandle,
    finished: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(target_arch = "wasm32")]
impl TaskHandle {
    #[inline]
    pub fn abort(&self) {
        self.abort_handle.abort();
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(target_arch = "wasm32")]
pub fn spawn(task: impl futures::Future<Output = ()> + Send + 'static) -> TaskHandle {
    let (abort_handle, abort_registration) = futures::future::AbortHandle::new_pair();
    let finished = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let task_finished = finished.clone();

    wasm_bindgen_futures::spawn_local(async move {
        let _ = futures::future::Abortable::new(task, abort_registration).await;
        task_finished.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    TaskHandle {
        abort_handle,
        finished,
    }
}

#[cfg(not(feature = "wasm-send"))]
pub trait Boxable<'a>: futures::Future + Sized + 'a {
    fn boxed(self) -> BoxFuture<'a, Self::Output> {
        futures::FutureExt::boxed_local(self)
    }
}
#[cfg(not(feature = "wasm-send"))]
impl<'a, F: futures::Future + Sized + 'a> Boxable<'a> for F {}

#[cfg(feature = "wasm-send")]
pub trait Boxable<'a>: futures::Future + Sized + 'a {
    fn boxed(self) -> BoxFuture<'a, Self::Output> {
        // tower-lsp requires Send futures even though a Web Worker has a single thread.
        // SendWrapper preserves that API contract and checks at runtime that the future
        // is only polled and dropped on the worker thread where it was created.
        futures::FutureExt::boxed(send_wrapper::SendWrapper::new(self))
    }
}

#[cfg(feature = "wasm-send")]
impl<'a, F: futures::Future + Sized + 'a> Boxable<'a> for F {}
