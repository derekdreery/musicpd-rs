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

/// This type can have its response tested for
pub trait ParseResponse {
    type ResponseType;
    /// Parse a response using nom's IResult
    fn parse_response<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], Self::ResponseType>;
}

/// Parses a line from the server into a version
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

/// Parses an mpd error
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

/// Parses the tag "OK\n". This tag is at the end of every successful response
named!(pub parse_ok, tag!(b"OK\n"));

/// Parses the tag "list_OK\n". This tag is at the end of every line in a successful response if
/// the list is started with "command_list_ok_begin".
named!(pub parse_list_ok, tag!(b"list_OK\n"));

/// Parses a number "0" or "1" and converts it to a bool. This is how booleans are transmitted.
named!(pub parse_num_bool<bool>, alt!(
    map!(tag!(b"0"), |_| false) |
    map!(tag!(b"1"), |_| true)
));

enum Sign {
    Pos,
    Neg
}

named!(parse_sign<Sign>,
    do_parse!(
        sign: one_of!(b"+-") >>
        (match sign {
            '+' => Sign::Pos,
            '-' => Sign::Neg,
            _ => unreachable!()
        })
    )
);

// NAIVE but good enough
// currently unused
named!(pub parse_f32<f32>,
    do_parse!(
        sign: opt!(parse_sign) >>
        whole: digit >>
        tag!(b".") >>
        frac: digit >>
        ({
            let sign = match sign {
                Some(Sign::Neg) => -1.0,
                _ => 1.0
            };
            let whole = str::from_utf8(whole).unwrap().parse::<f32>().unwrap();
            let frac_len = frac.len();
            let frac = str::from_utf8(frac).unwrap().parse::<f32>().unwrap();
            sign * (whole + frac * (10.0f32).powi(-(frac_len as i32)))
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

    #[test]
    fn num_bool() {
        let i = b"0";
        assert_eq!(
            parse_num_bool(&i[..]),
            IResult::Done(&b""[..], false)
        );
        let i = b"1";
        assert_eq!(
            parse_num_bool(&i[..]),
            IResult::Done(&b""[..], true)
        );
    }

    #[test]
    fn f32() {
        assert_eq!(
            parse_f32(&b"3.141"[..]),
            IResult::Done(&b""[..], 3.141)
        )
    }
}
