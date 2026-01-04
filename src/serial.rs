use std::time::Duration;

use futures::{
    SinkExt,
    channel::mpsc::{self, TrySendError},
    stream,
};
pub struct Uart {
    #[cfg(feature = "tokio")]
    rx: mpsc::Receiver<u8>,
    tx: mpsc::Sender<u8>,
    #[cfg(feature = "tokio")]
    timeout: Duration,
}

pub type UartStimulus = Uart;

impl Uart {
    pub fn new(
        #[allow(unused_variables)] timeout: Duration,
        buffer_size: usize,
    ) -> (Uart, UartStimulus) {
        let (rx_sender, _rx_receiver) = mpsc::channel(buffer_size);
        let (tx_sender, _tx_receiver) = mpsc::channel(buffer_size);

        (
            Uart {
                #[cfg(feature = "tokio")]
                rx: _rx_receiver,
                tx: tx_sender,
                #[cfg(feature = "tokio")]
                timeout,
            },
            UartStimulus {
                #[cfg(feature = "tokio")]
                rx: _tx_receiver,
                tx: rx_sender,
                #[cfg(feature = "tokio")]
                timeout,
            },
        )
    }

    #[cfg(feature = "tokio")]
    pub async fn read_until_idle(&mut self, dst: &mut [u8]) -> usize {
        use futures::StreamExt;
        use std::pin::pin;

        // TODO: consider ways to avoid the requirement on tokio
        for (i, byte) in dst.iter_mut().enumerate() {
            let t = pin!(tokio::time::sleep(self.timeout));
            tokio::select! {
                b = self.rx.next() => *byte = b.unwrap(),
                _ = t => return i,
            }
        }
        dst.len()
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
