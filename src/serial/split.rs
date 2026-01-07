#[cfg(any(feature = "tokio", target_arch = "wasm32"))]
use std::convert::Infallible;
use std::time::Duration;

use futures::{
    SinkExt,
    channel::mpsc::{self, TrySendError},
    stream,
};

/// Create two virtual serial interfaces that are connected to each other
///
/// What is sent from one interface can be received on the other etc
pub fn new(#[allow(unused_variables)] timeout: Duration, buffer_size: usize) -> (Split, Split) {
    let (rx_sender, _rx_receiver) = mpsc::channel(buffer_size);
    let (tx_sender, _tx_receiver) = mpsc::channel(buffer_size);

    (
        Split {
            #[cfg(any(feature = "tokio", target_arch = "wasm32"))]
            rx: _rx_receiver,
            tx: tx_sender,
            #[cfg(any(feature = "tokio", target_arch = "wasm32"))]
            timeout,
        },
        Split {
            #[cfg(any(feature = "tokio", target_arch = "wasm32"))]
            rx: _tx_receiver,
            tx: rx_sender,
            #[cfg(any(feature = "tokio", target_arch = "wasm32"))]
            timeout,
        },
    )
}

/// [new] create two virtual serial interfaces ([Split], [Split]) that are connected to each other
///
/// What is sent from one interface can be received on the other etc
pub struct Split {
    #[cfg(any(feature = "tokio", target_arch = "wasm32"))]
    rx: mpsc::Receiver<u8>,
    tx: mpsc::Sender<u8>,
    #[cfg(any(feature = "tokio", target_arch = "wasm32"))]
    timeout: Duration,
}

impl Split {
    #[cfg(any(feature = "tokio", target_arch = "wasm32"))]
    pub async fn read_until_idle(&mut self, dst: &mut [u8]) -> Result<usize, Infallible> {
        use futures::StreamExt;
        use std::pin::pin;

        // TODO: consider ways to avoid the requirement on tokio
        for (i, byte) in dst.iter_mut().enumerate() {
            use futures::FutureExt;

            let t = pin!(crate::sleep(self.timeout));
            futures::select! {
                b = self.rx.next().fuse() => *byte = b.unwrap(),
                _ = t.fuse() => return Ok(i),
            }
        }
        Ok(dst.len())
    }

    pub async fn write(&mut self, src: &[u8]) {
        let mut stream = stream::iter(src.iter().copied().map(Ok));
        self.tx.send_all(&mut stream).await.unwrap();
    }

    pub fn try_write(&mut self, src: &[u8]) -> Result<(), TrySendError<u8>> {
        for byte in src {
            self.tx.try_send(*byte)?;
        }

        Ok(())
    }
}
