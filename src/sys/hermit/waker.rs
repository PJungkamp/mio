use crate::sys::Selector;
use crate::{Interest, Token};

use std::io;
use std::os::hermit::abi;
use std::os::hermit::io::FromAbi;

#[derive(Debug)]
pub struct Waker {
    socket: abi::net::Socket,
}

impl Waker {
    pub fn new(selector: &Selector, token: Token) -> io::Result<Waker> {
        abi::net::socket()
            .and_then(|socket| { 
                abi::net::waker_bind(socket)?;
                Ok(socket)
            })
            .map_err(|err| unsafe { 
                io::Error::from_abi(err) 
            })
            .and_then(|socket| {
                selector
                    .register(socket, token, Interest::READABLE)
                    .map(|()| Waker { socket })
            })
    }

    pub fn wake(&self) -> io::Result<()> {
        abi::net::waker_send_event(self.socket,
            abi::net::event::EventFlags(
                abi::net::event::EventFlags::READABLE))
            .map_err(|err| unsafe { 
                io::Error::from_abi(err) 
            })
    }
}
