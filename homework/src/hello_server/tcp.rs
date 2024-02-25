//! TcpListener that can be cancelled.

use std::io;
use std::net::ToSocketAddrs;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};

/// Like `std::net::tcp::TcpListener`, but `cancel`lable.
#[derive(Debug)]
pub struct CancellableTcpListener {
    inner: TcpListener,

    /// An atomic boolean flag that indicates if the listener is `cancel`led.
    ///
    /// NOTE: This can be safely read/written by multiple thread at the same time (note that its
    /// methods take `&self` instead of `&mut self`). To set the flag, use `store` method with
    /// `Ordering::Release`. To read the flag, use `load` method with `Ordering::Acquire`. We  will
    /// discuss their precise semantics later.
    is_canceled: AtomicBool,
}

/// Like `std::net::tcp::Incoming`, but stops `accept`ing connections if the listener is `cancel`ed.
#[derive(Debug)]
pub struct Incoming<'a> {
    listener: &'a CancellableTcpListener,
}

impl CancellableTcpListener {
    /// Wraps `TcpListener::bind`.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<CancellableTcpListener> {
        let listener = TcpListener::bind(addr)?;
        Ok(CancellableTcpListener {
            inner: listener,
            is_canceled: AtomicBool::new(false),
        })
    }

    /// Signals the listener to stop accepting new connections.
    pub fn cancel(&self) -> io::Result<()> {
        // Set the flag first and make a bogus connection to itself to wake up the listener blocked
        // in `accept`. Use `TcpListener::local_addr` and `TcpStream::connect`.
        todo!()
    }

    /// Returns an iterator over the connections being received on this listener.  The returned
    /// iterator will return `None` if the listener is `cancel`led.
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }
}

impl Iterator for Incoming<'_> {
    type Item = io::Result<TcpStream>;
    /// Returns None if the listener is `cancel()`led.
    fn next(&mut self) -> Option<Self::Item> {
        let stream = self.listener.inner.accept().map(|p| p.0);
        todo!()
    }
}
