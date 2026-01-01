use embassy_net_driver_channel::{Runner, driver};

pub use embassy_net_driver_channel::Device;
pub struct EthernetBackgroundRunner<'d, const MTU: usize> {
    iface: tokio_tun::Tun,
    runner: Runner<'d, MTU>,
}

pub fn new<'d, const MTU: usize, const N_RX: usize, const N_TX: usize>(
    state: &'d mut embassy_net_driver_channel::State<MTU, N_RX, N_TX>,
    hardware_address: driver::HardwareAddress,
) -> (Device<'d, MTU>, EthernetBackgroundRunner<'d, MTU>) {
    let iface = tokio_tun::Tun::builder()
        .name("") // if name is empty, then it is set by kernel.
        .tap() // uses TAP instead of TUN (default).
        .packet_info() // avoids setting IFF_NO_PI.
        .up() // or set it up manually using `sudo ip link set <tun-name> up`.
        .close_on_exec() // or no_close_on_exec()
        .build()
        .unwrap()
        .pop()
        .unwrap();

    let (runner, device) = embassy_net_driver_channel::new(state, hardware_address);
    let runner = EthernetBackgroundRunner { runner, iface };
    (device, runner)
}

impl<'d, const MTU: usize> EthernetBackgroundRunner<'d, MTU> {
    /// This needs to be run from a tokio executor
    pub async fn run(self) {
        let (_s, mut rx_buff, mut tx_buff) = self.runner.split();
        let iface = self.iface;

        loop {
            // Wait for either..
            tokio::select! {
                n = async {
                    // the buffer is ready...
                    let buf = rx_buff.rx_buf().await;
                    // ... *AND* the interface is signaling an interrupt, indicating a packet is available to receive
                    iface.recv(buf).await.unwrap()
                } => {
                    rx_buff.rx_done(n)
                }
                // ... or a TX buffer becoming available, i.e. embassy-net wants to send a packet
                buf = tx_buff.tx_buf() => {
                    // a packet is ready to be sent!
                    iface.send_all(buf).await.unwrap();
                    tx_buff.tx_done();
                }
            }
        }
    }
}
