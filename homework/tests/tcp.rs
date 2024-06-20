use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::sync::mpsc::channel;
use std::thread::scope;
use std::time::Duration;

use cs431_homework::hello_server::CancellableTcpListener;

#[test]
fn cancellable_listener_cancel() {
    let mut port = 23456;
    let (addr, listener) = loop {
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port));
        if let Ok(listener) = CancellableTcpListener::bind(addr) {
            break (addr, listener);
        }
        port += 1;
    };

    let (done_sender, done_receiver) = channel();
    scope(|s| {
        let _ = s.spawn(|| {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut buf = [0];
                let _ = stream.read(&mut buf).unwrap();
                assert_eq!(buf[0], 123);
            }
            done_sender.send(()).unwrap();
        });
        let mut stream = TcpStream::connect(addr).unwrap();
        let _ = stream.write(&[123]).unwrap();

        listener.cancel().unwrap();
        done_receiver.recv_timeout(Duration::from_secs(3)).unwrap();
    });
}
