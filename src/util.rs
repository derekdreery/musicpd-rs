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
