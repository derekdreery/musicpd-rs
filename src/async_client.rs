use std::io;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};
use futures::{Future, Poll};

pub fn default_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 6600))
}

pub struct TokioMpc {
    stream: TcpStream
}

pub struct TokioMpcNew(TcpStreamNew);

impl Future for TokioMpcNew {
    type Item = TokioMpc;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll().map(|ok| ok.map(|stream| TokioMpc { stream: stream }))
    }
}

impl TokioMpc {
    pub fn new(addr: &SocketAddr, handle: &Handle) -> TokioMpcNew {
        TokioMpcNew(TcpStream::connect(addr, handle))
    }
}
