use std::{pin::pin, time::Duration};

use futures::{SinkExt, StreamExt, channel::mpsc, stream};
use tokio::select;

pub struct Uart {
    rx: mpsc::Receiver<u8>,
    tx: mpsc::Sender<u8>,
    timeout: Duration,
}

pub type UartStimulus = Uart;

impl Uart {
    pub fn new(timeout: Duration, buffer_size: usize) -> (Uart, UartStimulus) {
        let (rx_sender, rx_receiver) = mpsc::channel(buffer_size);
        let (tx_sender, tx_receiver) = mpsc::channel(buffer_size);

        (
            Uart {
                rx: rx_receiver,
                tx: tx_sender,
                timeout,
            },
            UartStimulus {
                rx: tx_receiver,
                tx: rx_sender,
                timeout,
            },
        )
    }

    pub async fn read_until_idle(&mut self, dst: &mut [u8]) -> usize {
        for (i, byte) in dst.iter_mut().enumerate() {
            let t = pin!(tokio::time::sleep(self.timeout));
            select! {
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
}
