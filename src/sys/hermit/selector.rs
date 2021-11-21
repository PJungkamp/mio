use log::error;
use crate::{Token,Interest};
use std::os::hermit::abi;
use std::os::hermit::io::{AsAbi, FromAbi};
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use std::io;

/// Unique id for use as `SelectorId`.
#[cfg(debug_assertions)]
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct Selector {
    #[cfg(debug_assertions)]
    id: usize,
    event_socket: abi::net::Socket,
    #[cfg(debug_assertions)]
    has_waker: AtomicBool,
}

impl Selector {
    pub fn new() -> io::Result<Selector> {
        abi::net::socket()
            .and_then(|socket| {
                abi::net::event_bind(socket)?;
                Ok(socket)
            })
            .map(|event_socket| Selector {
                #[cfg(debug_assertions)]
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                event_socket,
                #[cfg(debug_assertions)]
                has_waker: AtomicBool::new(false),
            })
            .map_err(|err| unsafe { io::Error::from_abi(err) })
    }

    pub fn try_clone(&self) -> io::Result<Selector> {
        unimplemented!()
    }

    pub fn select(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        events.clear();
        abi::net::socket_set_timeout(self.event_socket,timeout)
            .and_then(|_| unsafe {
                    abi::net::event_wait(
                        self.event_socket,
                        std::slice::from_raw_parts_mut(
                            events.as_mut_ptr() as *mut _,
                            events.capacity(),
                        )
                    )
            })
            .map(|n_events| unsafe { 
                events.set_len(n_events as usize) 
            })
            .map_err(|err| unsafe { io::Error::from_abi(err) })
    }

    pub fn register(&self, socket: abi::net::Socket, token: Token, interests: Interest) -> io::Result<()> {
        let event = Event {
            flags: interests_to_flags(interests),
            socket,
            data: token.0 as u64,
        };

        abi::net::event_add(self.event_socket, event)
            .map_err(|err| unsafe { io::Error::from_abi(err) })
    }

    pub fn reregister(&self, socket: abi::net::Socket, token: Token, interests: Interest) -> io::Result<()> {
        let event = Event {
            flags: interests_to_flags(interests),
            socket,
            data: token.0 as u64,
        };

        abi::net::event_modify(self.event_socket, event)
            .map_err(|err| unsafe { io::Error::from_abi(err) })
    }

    pub fn deregister(&self, socket: abi::net::Socket) -> io::Result<()> {
        abi::net::event_remove(self.event_socket, socket)
            .map_err(|err| unsafe { io::Error::from_abi(err) })
    }

    #[cfg(debug_assertions)]
    pub fn register_waker(&self) -> bool {
        self.has_waker.swap(true, Ordering::AcqRel)
    }
}

cfg_io_source! {
    impl Selector {
        #[cfg(debug_assertions)]
        pub fn id(&self) -> usize {
            self.id
        }
    }
}

impl AsAbi for Selector {
    type AbiType = abi::net::Socket;
    fn as_abi(&self) -> Self::AbiType {
        self.event_socket
    }
}

impl Drop for Selector {
    fn drop(&mut self) {
        abi::net::socket_close(self.event_socket)
            .unwrap_or_else(|err| error!("IO Error on close: {:?}",err))
    }
}

fn interests_to_flags(interests: Interest) -> abi::net::event::EventFlags {
    let mut flags = abi::net::event::EventFlags::NONE;

    if interests.is_readable() {
        flags |= abi::net::event::EventFlags::READABLE;
        flags |= abi::net::event::EventFlags::RCLOSED;
    }

    if interests.is_writable() {
        flags |= abi::net::event::EventFlags::WRITABLE;
        flags |= abi::net::event::EventFlags::WCLOSED;
    }

    abi::net::event::EventFlags(flags)
}

pub type Event = abi::net::event::Event;
pub type Events = Vec<Event>;

pub mod event {
    use std::fmt;
    use std::os::hermit::abi;
    use crate::sys::Event;
    use crate::Token;

    pub fn token(event: &Event) -> Token {
        Token(event.data as usize)
    }

    pub fn is_readable(event: &Event) -> bool {
        (event.flags.0 & abi::net::event::EventFlags::READABLE) != 0
    }

    pub fn is_writable(event: &Event) -> bool {
        (event.flags.0 & abi::net::event::EventFlags::WRITABLE) != 0
    }

    pub fn is_error(_: &Event) -> bool {
        false
    }

    pub fn is_read_closed(event: &Event) -> bool {
        (event.flags.0 & abi::net::event::EventFlags::RCLOSED) != 0
    }

    pub fn is_write_closed(event: &Event) -> bool {
        (event.flags.0 & abi::net::event::EventFlags::WCLOSED) != 0
    }

    pub fn is_priority(_: &Event) -> bool {
        false
    }

    pub fn is_aio(_: &Event) -> bool {
        false
    }

    pub fn is_lio(_: &Event) -> bool {
        false
    }

    pub fn debug_details(f: &mut fmt::Formatter<'_>, event: &Event) -> fmt::Result {
        f.debug_struct("Event")
            .field("readable",&is_readable(event))
            .field("writable",&is_writable(event))
            .field("rclosed",&is_read_closed(event))
            .field("wclosed",&is_write_closed(event))
            .field("socket", &event.socket)
            .field("data", &event.data)
            .finish()
    }
}
