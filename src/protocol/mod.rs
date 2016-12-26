pub mod command;

use std::str;
use std::io;

use nom::*;
use semver::Version;

use types::*;

/// Means that the object knows how to be serialized to a bytestream to be sent to the mpd server
pub trait Dispatch {
    /// Write out the current object to the mpd connection
    fn dispatch(&self, w: &mut io::Write) -> io::Result<()>;
}

named!(pub parse_handshake<Version>,
    do_parse!(
        tag!(b"OK MPD ") >>
        v: map_res!(
            map_res!(
                not_line_ending,
                str::from_utf8
            ),
            Version::parse
        ) >>
        line_ending >>
        (v)
    )
);

named!(pub parse_error<CmdError>,
    do_parse!(
        tag!(b"ACK [") >>
        code: map_opt!(
            digit,
            CmdErrorType::from_code
        ) >>
        tag!(b"@") >>
        index: map_opt!(
            digit,
            |raw| match str::from_utf8(raw) {
                Ok(s) => match str::parse::<usize>(s) {
                    Ok(val) => Some(val),
                    Err(_) => None
                },
                Err(_) => None
            }
        ) >>
        tag!(b"] {") >>
        name: map_res!(
            is_not!("}\r\n"),
            str::from_utf8
        ) >>
        tag!(b"} ") >>
        message: map_res!(
            not_line_ending,
            str::from_utf8
        ) >>
        line_ending >>
        (CmdError {
            error_type: code,
            command_no: index,
            command_name: name.to_owned(),
            message_text: message.to_owned()
        })
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use nom::*;
    use semver::Version;
    use types::*;

    #[test]
    fn handshake() {
        let i = b"OK MPD 0.12.2\n";
        assert_eq!(
            parse_handshake(&i[..]),
            IResult::Done(&b""[..], Version::parse("0.12.2").unwrap())
        );
    }

    #[test]
    fn error() {
        let i = b"ACK [50@1] {play} song doesn't exist: \"10240\"\n";
        assert_eq!(
            parse_error(&i[..]),
            IResult::Done(&b""[..], CmdError {
                error_type: CmdErrorType::NoExist,
                command_no: 1,
                command_name: "play".to_owned(),
                message_text: "song doesn't exist: \"10240\"".to_owned()
            })
        );
    }
}
