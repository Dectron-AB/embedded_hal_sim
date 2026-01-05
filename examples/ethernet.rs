use std::time::Duration;

use embassy_net::driver::HardwareAddress;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, StackResources};
use embassy_net_driver_channel::{Device, State};
use embedded_hal_sim::ethernet::{self, EthernetBackgroundRunner};
use embedded_io_async::Write;
use log::info;
use simple_logger::SimpleLogger;
use static_cell::StaticCell;

async fn ethernet_task(runner: EthernetBackgroundRunner<'static, MTU>) -> ! {
    log::trace!("Ethernet task started");
    runner.run().await
}

async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static, MTU>>) -> ! {
    log::trace!("Net task started");
    runner.run().await
}

/// The MTU used in the embassy-net-wiznet
const MTU: usize = 1514;

#[tokio::main(flavor = "local")]
async fn main() -> ! {
    SimpleLogger::new().init().unwrap();

    info!("Hello World!");

    embassy_time::Timer::after_secs(1).await;

    info!("Waiting works!");

    // Generate random seed
    let seed = rand::random();

    let mac_addr = [0xa6, 0x39, 0xf0, 0xbd, 0x03, 0xf0];
    static STATE: StaticCell<State<MTU, 2, 2>> = StaticCell::new();
    let state = STATE.init(State::<MTU, 2, 2>::new());

    info!("Creating ethernet interface");
    let (device, runner) = ethernet::new(state, HardwareAddress::Ethernet(mac_addr));
    info!("Spawn ethernet task");
    tokio::task::spawn_local(ethernet_task(runner));

    info!("Configure dhcp");
    let config = embassy_net::Config::dhcpv4(Default::default());
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    //});

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    info!("Setup stack");
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    info!("Spawn net stack");
    // Launch network task
    tokio::task::spawn_local(net_task(runner));

    info!("Await config up");

    // Ensure DHCP configuration is up before trying connect
    stack.wait_config_up().await;

    info!("Network task initialized");

    // Then we can use it!
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Address::new(10, 42, 0, 1), 8000);
        info!("connecting...");
        let r = socket.connect(remote_endpoint).await;
        if let Err(e) = r {
            info!("connect error: {:?}", e);
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        info!("connected!");
        let buf = [0; 1024];
        loop {
            let r = socket.write_all(&buf).await;
            if let Err(e) = r {
                info!("write error: {:?}", e);
                break;
            }
            //Timer::after_secs(1).await;
        }
    }
}
