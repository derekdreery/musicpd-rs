use std::net;
use std::io;
use std::io::prelude::*;
use std::error::Error as StdError;
use std::fmt::Debug;

use semver::Version;
use nom::{IResult, ErrorKind};
#[cfg(feature = "verbose-errors")]
use nom::Err as NomErr;

use protocol::command::{CommandList, CommandResponse};
use protocol::{Dispatch, ParseResponse, parse_handshake,};
use util::Buffer;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    #[cfg(not(feature = "verbose-errors"))]
    Parse(ErrorKind),
    #[cfg(feature = "verbose-errors")]
    Parse(Box<StdError>),
}

// use a buffered reader, but get inner for writes
pub struct Client {
    stream: io::BufReader<net::TcpStream>,
    version: Version
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

#[cfg(not(feature = "verbose-errors"))]
impl From<ErrorKind> for Error {
    fn from(e: ErrorKind) -> Self {
        Error::Parse(e)
    }
}

#[cfg(feature = "verbose-errors")]
impl<P: Debug + 'static> From<NomErr<P>> for Error {
    fn from(e: NomErr<P>) -> Self {
        Error::Parse(Box::new(e))
    }
}

impl Client {
    pub fn connect<A: net::ToSocketAddrs>(addr: A) -> Result<Client, Error> {
        let mut stream = io::BufReader::new(net::TcpStream::connect(addr)?);
        let version = match Buffer::parse(parse_handshake, &mut stream) {
            IResult::Done(_, v) => v,
            IResult::Incomplete(_) => unreachable!(),
            IResult::Error(e) => { return Err(Error::from(e)) }
        };
        Ok(Client {
            stream: stream,
            version: version
        })
    }

    pub fn version(&self) -> Version {
        self.version.clone()
    }

    pub fn run_commands(&mut self, commands: CommandList)
        -> Result<Vec<CommandResponse>, Error>
    {
        commands.dispatch(&mut self.stream.get_mut());
        let response = match Buffer::parse(|i| commands.parse_response(i), &mut self.stream) {
            IResult::Done(_, v) => v,
            IResult::Incomplete(_) => unreachable!(),
            IResult::Error(e) => { return Err(Error::from(e)) }
        };
        println!("{:?}", response);
        Ok(Vec::new())
    }
}
