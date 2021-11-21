// Helper macro to execute a system call that returns an `io::Result`.
//
// Macro must be defined before any modules that uses them.

cfg_os_poll! {
    mod selector;
    pub(crate) use self::selector::{event, Event, Events, Selector};

    mod waker;
    pub(crate) use self::waker::Waker;

    cfg_net! {
        pub(crate) mod tcp;
        pub(crate) mod udp;
    }

    cfg_io_source! {
        use std::io;

        // No userspace state required
        pub(crate) struct IoSourceState;

        impl IoSourceState {
            pub fn new() -> IoSourceState {
                IoSourceState
            }

            pub fn do_io<T, F, R>(&self, f: F, io: &T) -> io::Result<R>
            where
                F: FnOnce(&T) -> io::Result<R>,
            {
                // We don't hold state, so we can just call the function and
                // return.
                f(io)
            }
        }
    }
}
