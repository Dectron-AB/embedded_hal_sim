#[cfg(not(feature = "tokio"))]
use std::sync::Arc;

pub fn signal<T: Clone>(_init: T) -> (SignalTx<T>, SignalRx<T>) {
    #[cfg(feature = "tokio")]
    let (tx, rx) = tokio::sync::watch::channel(_init);

    #[cfg(not(feature = "tokio"))]
    let (tx, rx) = {
        let tx = Arc::new(embassy_sync::signal::Signal::new());
        let rx = Arc::clone(&tx);
        (tx, rx)
    };

    (SignalTx { inner: tx }, SignalRx { inner: rx })
}

pub struct SignalRx<T: Clone> {
    #[cfg(feature = "tokio")]
    inner: tokio::sync::watch::Receiver<T>,

    #[cfg(not(feature = "tokio"))]
    inner: Arc<embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::NoopRawMutex, T>>,
}

impl<T: Clone> SignalRx<T> {
    pub async fn wait(&mut self) -> T {
        #[cfg(feature = "tokio")]
        return self.inner.wait_for(|_| true).await.unwrap().clone();

        #[cfg(not(feature = "tokio"))]
        self.inner.wait().await
    }
}

pub struct SignalTx<T: Clone> {
    #[cfg(feature = "tokio")]
    inner: tokio::sync::watch::Sender<T>,

    #[cfg(not(feature = "tokio"))]
    inner: Arc<embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::NoopRawMutex, T>>,
}

impl<T: Clone> SignalTx<T> {
    pub fn signal(&mut self, x: T) {
        #[cfg(feature = "tokio")]
        return self.inner.send(x).unwrap();

        #[cfg(not(feature = "tokio"))]
        self.inner.signal(x)
    }
}
