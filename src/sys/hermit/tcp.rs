use std::io;
use std::net::{self, SocketAddr};
use std::os::hermit::io::{AsAbi, FromAbi};
use std::os::hermit::abi;

pub(crate) fn new_for_addr(_address: SocketAddr) -> io::Result<abi::net::Socket> {
    abi::net::socket()
        .and_then(|socket| { 
            abi::net::socket_set_non_blocking(socket,true)?;
            Ok(socket)
        })
        .map_err(|err| unsafe { io::Error::from_abi(err) })
}

pub(crate) fn bind(socket: &net::TcpListener, addr: SocketAddr) -> io::Result<()> {
    abi::net::tcp_bind(socket.as_abi(), addr.as_abi())
        .map_err(|err| unsafe { io::Error::from_abi(err) })
}

pub(crate) fn connect(socket: &net::TcpStream, addr: SocketAddr) -> io::Result<()> {
    abi::net::tcp_bind(socket.as_abi(), 
        abi::net::SocketAddr::V4(abi::net::SocketAddrV4::UNSPECIFIED))
        .map_err(|err| unsafe { io::Error::from_abi(err) })?;
    abi::net::tcp_connect(socket.as_abi(), addr.as_abi())
        .map_err(|err| unsafe { io::Error::from_abi(err) })
}

pub(crate) fn listen(socket: &net::TcpListener, backlog: u32) -> io::Result<()> {
    abi::net::tcp_listen(socket.as_abi(), backlog as usize)
        .map_err(|err| unsafe { io::Error::from_abi(err) })
}

pub(crate) fn accept(listener: &net::TcpListener) -> io::Result<(net::TcpStream, SocketAddr)> {
    let socket = abi::net::tcp_accept(listener.as_abi())
        .map_err(|err| unsafe { io::Error::from_abi(err) })?;
    let stream = unsafe { net::TcpStream::from_abi(socket) };
    let peer = stream.peer_addr()?;
    Ok((stream,peer))
}
