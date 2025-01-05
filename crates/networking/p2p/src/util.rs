/// TODO: this file contains code from lighthouse. crit!("re-implement")
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use libp2p::{multiaddr::Protocol, Multiaddr};
use serde::{Deserialize, Serialize};

/// A listening address composed by an Ip, an UDP port and a TCP port.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListenAddr<Ip> {
    /// The IP address we will listen on.
    pub addr: Ip,
    /// The UDP port that discovery will listen on.
    pub disc_port: u16,
    /// The UDP port that QUIC will listen on.
    pub quic_port: u16,
    /// The TCP port that libp2p will listen on.
    pub tcp_port: u16,
}

impl<Ip: Into<IpAddr> + Clone> ListenAddr<Ip> {
    pub fn discovery_socket_addr(&self) -> SocketAddr {
        (self.addr.clone().into(), self.disc_port).into()
    }

    pub fn quic_socket_addr(&self) -> SocketAddr {
        (self.addr.clone().into(), self.quic_port).into()
    }

    pub fn tcp_socket_addr(&self) -> SocketAddr {
        (self.addr.clone().into(), self.tcp_port).into()
    }
}

/// Types of listening addresses Lighthouse can accept.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ListenAddress {
    V4(ListenAddr<Ipv4Addr>),
    V6(ListenAddr<Ipv6Addr>),
    DualStack(ListenAddr<Ipv4Addr>, ListenAddr<Ipv6Addr>),
}

impl ListenAddress {
    /// Return the listening address over IpV4 if any.
    pub fn v4(&self) -> Option<&ListenAddr<Ipv4Addr>> {
        match self {
            ListenAddress::V4(v4_addr) | ListenAddress::DualStack(v4_addr, _) => Some(v4_addr),
            ListenAddress::V6(_) => None,
        }
    }

    /// Return the listening address over IpV6 if any.
    pub fn v6(&self) -> Option<&ListenAddr<Ipv6Addr>> {
        match self {
            ListenAddress::V6(v6_addr) | ListenAddress::DualStack(_, v6_addr) => Some(v6_addr),
            ListenAddress::V4(_) => None,
        }
    }

    /// Returns the addresses the Swarm will listen on, given the setup.
    pub fn libp2p_addresses(&self) -> impl Iterator<Item = Multiaddr> {
        let v4_tcp_multiaddr = self
            .v4()
            .map(|v4_addr| Multiaddr::from(v4_addr.addr).with(Protocol::Tcp(v4_addr.tcp_port)));

        let v4_quic_multiaddr = self.v4().map(|v4_addr| {
            Multiaddr::from(v4_addr.addr)
                .with(Protocol::Udp(v4_addr.quic_port))
                .with(Protocol::QuicV1)
        });

        let v6_quic_multiaddr = self.v6().map(|v6_addr| {
            Multiaddr::from(v6_addr.addr)
                .with(Protocol::Udp(v6_addr.quic_port))
                .with(Protocol::QuicV1)
        });

        let v6_tcp_multiaddr = self
            .v6()
            .map(|v6_addr| Multiaddr::from(v6_addr.addr).with(Protocol::Tcp(v6_addr.tcp_port)));

        v4_tcp_multiaddr
            .into_iter()
            .chain(v4_quic_multiaddr)
            .chain(v6_quic_multiaddr)
            .chain(v6_tcp_multiaddr)
    }
}
