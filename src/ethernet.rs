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
    pub async fn run(self) -> ! {
        let (state, mut rx_buff, mut tx_buff) = self.runner.split();
        state.set_link_state(driver::LinkState::Up);

        let iface = self.iface;

        let mut buf = [0u8; MTU];
        let mut bytes_available = 0;
        
        loop {
            // Wait for either..
            tokio::select! {
                rx_buf = async {
                    // the interface is signaling an interrupt, indicating a packet is available to receive
                    if bytes_available == 0 {
                        bytes_available = iface.recv(&mut buf).await.unwrap();
                    }
                    // ... *AND* the buffer is ready...
                    rx_buff.rx_buf().await
                } => {
                    log::trace!("rx: {bytes_available}");
                    rx_buf[..bytes_available].copy_from_slice(&buf[..bytes_available]);
                    rx_buff.rx_done(bytes_available);
                    bytes_available = 0;
                }
                // ... or a TX buffer becoming available, i.e. embassy-net wants to send a packet
                buf = tx_buff.tx_buf() => {
                    log::trace!("tx: {}", buf.len());
                    // a packet is ready to be sent!
                    iface.send_all(buf).await.unwrap();
                    tx_buff.tx_done();
                }
            }
        }
    }
}
