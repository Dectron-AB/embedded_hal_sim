#[cfg(any(feature = "tokio", target_arch = "wasm32"))]
use std::convert::Infallible;
use std::{collections::VecDeque, time::Duration};

use eframe::wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Uint8Array;

use crate::utils::{SignalRx, SignalTx, signal};

pub struct Web {
    w: SignalRx<Option<web_sys::SerialPort>>,
    port: Option<web_sys::SerialPort>,
    buf: VecDeque<u8>,
    timeout: Duration,
}

pub struct SerialRequest {
    signal: SignalTx<Option<web_sys::SerialPort>>,
}

pub fn new(timeout: Duration) -> (Web, SerialRequest) {
    let (tx, rx) = signal(None);
    (
        Web {
            w: rx,
            port: None,
            buf: VecDeque::new(),
            timeout,
        },
        SerialRequest{ signal: tx },
    )
}

impl SerialRequest {
    /// Run this as the response of a button to request a serial port
    pub async fn request(mut self, options: &web_sys::SerialOptions) -> Result<(), Self> {
        let port: Result<web_sys::SerialPort, _> = JsFuture::from(
            web_sys::window()
                .unwrap()
                .navigator()
                .serial()
                .request_port(),
        )
        .await
        .unwrap()
        .dyn_into();

        let Ok(port) = port else {
            return Err(self);
        };

        JsFuture::from(port.open(options)).await.unwrap();

        self.signal.signal(Some(port));

        Ok(())
    }
}

// https://developer.mozilla.org/en-US/docs/Web/API/SerialPort
impl Web {
    async fn get_port(&mut self) -> &mut web_sys::SerialPort {
        // When the `Web` is new, it does not have a SerialPort
        // an opened port will be sent via self.w to `Web` once [SerialRequest::request]
        // finishes. We then store the port in self.port for further use
        match &mut self.port {
            Some(p) => return p,
            port => loop {
                *port = self.w.wait().await;
                if let Some(p) = port {
                    return p;
                }
            },
        }
    }

    #[cfg(any(feature = "tokio", target_arch = "wasm32"))]
    pub async fn read_until_idle(&mut self, dst: &mut [u8]) -> Result<usize, Infallible> {
        use eframe::wasm_bindgen::JsValue;
        use embassy_futures::select::select;
        use web_sys::{ReadableStreamDefaultReader, js_sys};

        let port = self.get_port().await;
        let r: ReadableStreamDefaultReader = port.readable().get_reader().dyn_into().unwrap();

        let read = || async {
            use web_sys::js_sys::Uint8Array;

            let value: Uint8Array = js_sys::Reflect::get(&r, &JsValue::from_str("value"))
                .unwrap()
                .dyn_into()
                .unwrap();
            let done: bool = js_sys::Reflect::get(&r, &JsValue::from_str("done"))
                .unwrap()
                .is_truthy();
            assert!(!done);

            value.to_vec()
        };

        let read = async {
            while self.buf.len() < dst.len() {
                self.buf.extend(read().await);
            }
        };
        let timeout = crate::sleep(self.timeout);
        select(read, timeout).await;

        let count = self.buf.len().min(dst.len());

        // Fill dst with the oldest data and remove it from self.buf
        for (d, src) in dst.iter_mut().zip(self.buf.drain(0..count)) {
            *d = src;
        }

        Ok(count)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/WritableStreamDefaultWriter
    pub async fn write(&mut self, src: &[u8]) {
        let port = self.get_port().await;
        let writer = port.writable().get_writer().unwrap();
        JsFuture::from(writer.ready()).await.unwrap();

        JsFuture::from(writer.write_with_chunk(&Uint8Array::new_from_slice(src)))
            .await
            .unwrap();
    }
}
