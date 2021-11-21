use std::io;
use std::net::{self,SocketAddr};

pub fn bind(_addr: SocketAddr) -> io::Result<net::UdpSocket> {
    unimplemented!()
}

pub(crate) fn only_v6(_socket: &net::UdpSocket) -> io::Result<bool> {
    unimplemented!()
}
