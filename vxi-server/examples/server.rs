use std::collections::BTreeMap;
use std::os::unix::io::AsRawFd;

use arrayvec::ArrayVec;
use log::debug;
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache};
use smoltcp::phy::wait as phy_wait;
use smoltcp::socket::{SocketHandle, SocketSet};
use smoltcp::socket::{TcpSocket, TcpSocketBuffer};
use smoltcp::socket::{UdpPacketMetadata, UdpSocket, UdpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};

use vxi_server::hislip::client::Client;
use vxi_server::hislip::Server;
use vxi_server::SocketPool;
use vxi_server::vxi11::portmapper::{PortMapper, Mapping, Iproto};
use vxi_server::vxi11::vxiservice::{Vxi11Server, Link};
use vxi_server::vxi11_service;

mod utils;

fn main() {
    utils::setup_logging("");

    let (mut opts, mut free) = utils::create_options();
    utils::add_tap_options(&mut opts, &mut free);
    utils::add_middleware_options(&mut opts, &mut free);

    let mut matches = utils::parse_options(&opts, free);
    let device = utils::parse_tap_options(&mut matches);
    let fd = device.as_raw_fd();
    let device = utils::parse_middleware_options(&mut matches, device, /*loopback=*/ false);

    let neighbor_cache = NeighborCache::new(BTreeMap::new());

    let ethernet_addr = EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);
    let ip_addrs = [
        IpCidr::new(IpAddress::v4(192, 168, 69, 1), 24),
        IpCidr::new(IpAddress::v6(0xfdaa, 0, 0, 0, 0, 0, 0, 1), 64),
        IpCidr::new(IpAddress::v6(0xfe80, 0, 0, 0, 0, 0, 0, 1), 64),
    ];
    let mut iface = EthernetInterfaceBuilder::new(device)
        .ethernet_addr(ethernet_addr)
        .neighbor_cache(neighbor_cache)
        .ip_addrs(ip_addrs)
        .finalize();

    let mut tcp_socket_pool: SocketPool<[SocketHandle; 20]> = SocketPool::new();
    let mut sockets = SocketSet::new(vec![]);

    for _ in 0..10 {
        let handle = sockets.add(TcpSocket::new(
            TcpSocketBuffer::new(vec![0; 512]),
            TcpSocketBuffer::new(vec![0; 512]),
        ));
        tcp_socket_pool.add_socket(handle);
    }
    let portmapper_udp = sockets.add(
        UdpSocket::new(
            UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 512]),
            UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 512])
        )
    );

    let mut hislip_server: Server<[SocketHandle; 20], [Client; 5]> = Server::new(4880, 0xBEEF);

    //let mut rpc_service: RpcService<[(SocketHandle, ArrayVec<[u8; 512]>); 10], [u8; 512]> =
    //    = RpcService::new(100000, 2, 111, None /*TODO*/, Some(portmapper_udp));
    let mut portmap_service: PortMapper<[Mapping; 512], [(SocketHandle, ArrayVec<[u8; 512]>); 10], [u8; 512]>
        = PortMapper::new(None /*TODO*/, Some(portmapper_udp));

    let mut vxi_service: vxi11_service!(2, 1024, 256) = Vxi11Server::new(None, None);

    portmap_service.register_self();
    portmap_service.set(0x0607AF, 1, Iproto::Tcp, 1024);
    portmap_service.set(0x0607B0, 1, Iproto::Tcp, 1025);

    loop {
        let timestamp = Instant::now();
        match iface.poll(&mut sockets, timestamp) {
            Ok(_) => {}
            Err(e) => {
                debug!("poll error: {}", e);
            }
        }

        hislip_server.serve(&mut tcp_socket_pool, &mut sockets);
        portmap_service.serve(&mut tcp_socket_pool, &mut sockets).unwrap();
        vxi_service.serve(&mut tcp_socket_pool, &mut sockets).unwrap();

        phy_wait(fd, iface.poll_delay(&sockets, timestamp)).expect("wait error");
    }
}
