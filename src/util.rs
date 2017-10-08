use std::io;
use std::slice;
use std::ops::{Index, RangeFull, RangeFrom};
use std::str;
use nom::*;
/*
/// Basically map a string error to a nom error
/// Unnecessary if not using `Needed`
fn map_str(i: &[u8]) -> IResult<&[u8], &str> {
    match str::from_utf8(i) {
        Ok(s) => IResult::Done(&i[i.len()..], s),
        Err(_) => IResult::Error(ErrorKind::Char)
    }
}
*/
/*
/// A simplified version of semver
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize
}

impl Version {
    pub fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self { major: major, minor: minor, patch: patch }
    }

    pub fn parse(&str) => Result<Self, ()> {
        match do_parse!(
            major: many1!(digit) >>
            tag!(b".") >>
            minor: many1!(digit) >>
            tag!(b".") >>
            patch: many1!(digit) >>
            (major, minor, patch)
        ) {
    }
}
*/

const DEFAULT_BLOCK_SIZE: usize = 512;

/// A buffer struct
///
/// This doesn't do certain things very well, in fact it doesn't really work, but
/// because of data coming in nice whole packets I think I don't need it to be any
/// better.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Buffer {
    buf: Vec<u8>,
    block_size: usize,
    pos: usize
}

impl Buffer {

    /// Create a new buffer
    #[inline]
    pub fn new() -> Buffer {
        Buffer::with_block_size(DEFAULT_BLOCK_SIZE)
    }

    /// Create a buffer that reallocates with given block size (underlying
    /// Vec may reallocate less often)
    pub fn with_block_size(block_size: usize) -> Buffer {
        Buffer {
            buf: Vec::with_capacity(block_size),
            block_size: block_size,
            pos: 0
        }
    }

    /// Do a read call and add to our vector
    ///
    /// Returns amount of new data added
    pub fn fetch<R>(&mut self, reader: &mut R) -> io::Result<usize>
    where R: io::Read {
        if self.buf.capacity() == self.buf.len() {
            self.buf.reserve(self.block_size);
        }

        let p = self.buf.as_mut_ptr();
        let len = self.buf.len();
        // check for overflow (is this necessary)
        assert!(len < isize::max_value() as usize);
        let capacity = self.buf.capacity();

        // create a slice from the unassigned part of the vec
        let extra = unsafe {
            slice::from_raw_parts_mut(
                p.offset(len as isize),
                capacity - len
            )
        };
        // if this fails we just leave vec as is (i.e. do nothing in ? branch)
        let amt = reader.read(extra)?;
        // safety check
        assert!(len + amt <= capacity);
        unsafe {
            // Adjust length to include new data
            self.buf.set_len(len + amt);
        }
        Ok(amt)
    }

    /// Parses from a read source, asks for more data if we hit an incomplete
    pub fn parse<F, R, O>(mut parser: F, mut reader: R)
        -> IResult<(), O>
        where F: FnMut(&[u8]) -> IResult<&[u8], O>,
        R: io::Read,
        O: Clone
    {
        let mut buf = Self::new();
        loop {
            // this intermediate variable is here for borrow-checker reasons
            let mut res = None;
            // TODO io error
            buf.fetch(&mut reader).unwrap();
            println!("try parse on **{}**", str::from_utf8(&buf[..]).unwrap());
            match parser(&buf[..]) {
                IResult::Done(i, o) => {
                    res = Some((i.len(), o));
                },
                IResult::Error(e) => {
                    return IResult::Error(e);
                },
                IResult::Incomplete(i) => {
                    //println!("got {:?}, carrying on", i);
                }
            }
            if let Some((amt, out)) = res {
                buf.pos += amt;
                return IResult::Done((), out);
            }
        }
    }
}

impl Index<RangeFull> for Buffer {
    type Output = [u8];
    fn index(&self, _: RangeFull) -> &[u8] {
        self.buf.index(RangeFrom {
            start: self.pos
        })
    }
}

#[test]
fn test_simple() {
    let data1 = vec![1u8, 0, 1];
    let data2 = vec![3u8, 4, 5];
    let mut b = Buffer::new();
    b.fetch(&mut &data1[..]).unwrap();
    assert_eq!(b.buf.len(), 3);
    assert_eq!(&b.buf[..], [1u8, 0, 1]);
    b.fetch(&mut &data2[..]).unwrap();
    assert_eq!(b.buf.len(), 6);
    assert_eq!(&b.buf[..], [1u8, 0, 1, 3, 4, 5]);
}

#[test]
fn test_with_realloc() {
    let data1 = vec![1u8, 0, 1];
    let data2 = vec![3u8, 4, 5];
    let mut b = Buffer::with_block_size(2);
    b.fetch(&mut &data1[..]).unwrap();
    assert_eq!(b.buf.len(), 2);
    assert_eq!(&b.buf[..], [1u8, 0]);
    b.fetch(&mut &data2[..]).unwrap();
    assert_eq!(b.buf.len(), 4);
    assert_eq!(&b.buf[..], [1u8, 0, 3, 4]);
}

/// Parse from bytes, rather than str
///
/// # Panics
/// Panics when byte sequence is not valid utf8
pub fn parse_bytes<F>(bytes: &[u8]) -> Result<F, F::Err>
    where F: str::FromStr
{
    let utf8 = str::from_utf8(bytes).unwrap();
    utf8.parse()
}

#[cfg(feature = "verbose-errors")]
fn dbg_process_error<O>(e: Err<&[u8]>) {
    panic!(format!("{}", err_map_str(e)))
}

#[cfg(not(feature = "verbose-errors"))]
fn dbg_process_error<O>(e: ErrorKind) -> IResult<(), O> {
    IResult::Error(e)
}

/// A small fn to help with error reporting
#[cfg(feature = "verbose-errors")]
fn err_map_str(e: Err<&[u8]>) -> String {
    match e {
        Err::Code(kind) => format!("Parser error with kind \"{:?}\"", kind).into(),
        Err::Node(kind, bx) => format!("Parser error with kind \"{:?}\"", kind).into(),
        Err::Position(kind, p) => format!(
            "Parser error with kind \"{:?}\" at {:?}",
            kind,
            str::from_utf8(p).unwrap_or("not utf8")
        ).into(),
        Err::NodePosition(kind, p, bx) => format!(
            "Parser error with kind \"{:?}\" at {:?}",
            kind,
            str::from_utf8(p).unwrap_or("not utf8")
        ).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}
