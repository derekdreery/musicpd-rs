use std::io;
use std::str;
use std::time::Duration;

use nom::*;
use chrono::{UTC, TimeZone};

use super::{Dispatch, ParseResponse, parse_ok, parse_list_ok, parse_num_bool, parse_f32};
use util::{parse_bytes};
use types::{SubSystem, ReplayGainMode, State, Status, MaybeStatus, Stats, MaybeStats,
    Range, SingleOrRange, TagType};

/// Of form name: value\n
macro_rules! parse_status_line (
    ($i: expr, $tag: expr) => (
        do_parse!($i,
            tag!($tag) >>
            tag!(b": ") >>
            out: not_line_ending >>
            tag!(b"\n") >>
            (out)
        )
    );
);

#[test]
fn test_parse_status_line() {
    let input = b"volume: 23\n";
    assert_eq!(
        map_res!(&input[..], parse_status_line!(b"volume"), parse_bytes::<u32>),
        IResult::Done(&b""[..], 23)
    );
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandList(Vec<Command>);

impl CommandList {
    /// Create an empty command list
    pub fn new() -> Self {
        CommandList(Vec::new())
    }

    pub fn push(&mut self, c: Command) {
        self.0.push(c);
    }
}

impl Dispatch for CommandList {
    /// Dispatch the command list to the server
    fn dispatch(&self, w: &mut io::Write) -> io::Result<()> {
        w.write_all(b"command_list_ok_begin\n")?;
        for cmd in &self.0 {
            cmd.dispatch(w)?;
            //println!("{:?}", cmd);
        }
        w.write_all(b"command_list_end\n")?;
        Ok(())
    }
}

impl ParseResponse for CommandList {
    type ResponseType = Vec<CommandResponse>;

    fn parse_response<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], Self::ResponseType> {
        let mut response = Vec::with_capacity(self.0.len());
        let mut i_inner = i;
        for cmd in &self.0 {
            let command_response = match cmd.parse_response(i_inner) {
                IResult::Done(i, res) => {
                    i_inner = i;
                    res
                },
                IResult::Error(e) => { return IResult::Error(e) },
                IResult::Incomplete(n) => { return IResult::Incomplete(n) }
            };
            response.push(command_response);
            let (i, _) = try_parse!(i_inner, parse_list_ok);
            i_inner = i;
        }
        let (i, _) = try_parse!(i_inner, dbg!(parse_ok));
        IResult::Done(i, response)
    }
}

/// All possible commands that can be sent to the server.
///
/// For some commands, a file path is specified. This can often either be a relative path from the
/// music directory, or a URI with supported scheme.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Clears the current error message in status (this is also accomplished by any command that
    /// starts playback).
    ClearError,
    /// Displays the song info of the current song (same song that is identified in status).
    CurrentSong,
    /// Waits until there is a noteworthy change in one or more of MPD's subsystems.
    ///
    /// If list of subsystems is empty, all subsystem changes are subscribed to
    Idle(Vec<SubSystem>),
    /// Reports the current status of the player and the volume level.
    Status,
    /// Displays statistics.
    Stats,
    /// Activates or deactivates consume.
    ///
    /// True to activate, false to deactivate.
    Consume(bool),
    /// Sets crossfading between songs to the given number of seconds.
    Crossfade(u16),
    /// Sets the threshold at which songs will be overlapped. Like crossfading but doesn't fade the
    /// track volume, just overlaps. The songs need to have MixRamp tags added by an external tool.
    /// 0dB is the normalized maximum volume so use negative values, I prefer -17dB. In the absence
    /// of mixramp tags crossfading will be used. See http://sourceforge.net/projects/mixramp
    MixRampDB(i16),
    /// Additional time subtracted from the overlap calculated by mixrampdb. If `None`, disables
    /// MixRamp overlapping and falls back to crossfading.
    MixRampDelay(Option<u16>),
    /// Sets the random (a.k.a shuffle) state. True to enable, false to disable.
    Random(bool),
    /// Sets the repeat state. True to enable, false to disable.
    Repeat(bool),
    /// Set the volume to the given value, clamped at 100 (called setvol)
    Volume(u8),
    /// Sets single state. When single is activated, playback is stopped after current song, or
    /// song is repeated if the 'repeat' mode is enabled. True to enable, false to disable
    Single(bool),
    /// Sets the replay gain mode.
    ReplayGainMode(ReplayGainMode),
    /// Fetches replay gain options.
    ReplayGainStatus,
    /// Play next song
    Next,
    /// Pause or unpause the playing track. True to pause, False to unpause. Pausing a paused track
    /// is a no-op and vice-verca
    Pause(bool),
    /// Move to song at given position on playlist and play.
    Play(u32),
    /// Move to song with given id and play.
    PlayId(String),
    /// Move to the previous song
    Previous,
    /// Plays from the given time in the song at the given position in the playlist
    Seek {
        song_position: u32,
        time: Duration,
    },
    /// Plays from the given time in the song with the given id
    SeekId {
        song_id: String,
        time: Duration,
    },
    /// Plays from the given time in the current song.
    SeekCurrent(Duration),
    /// Stop playing
    Stop,
    /// Adds the file at `uri` to the current playlist (directories add recursively).
    Add(String),
    /// Adds a song to the current playlist and returns the song id.
    ///
    /// Can also optionally add a position to insert into playlist.
    /// Adding a directory using this method will return an error.
    AddId {
        uri: String,
        position: Option<u32>
    },
    /// Clears the current playlist
    Clear,
    /// Deletes a song or range of songs from the playlist
    Delete(SingleOrRange),
    /// Deletes a song from playlist by ID
    DeleteId(String),
    /// Moves the song or range at `from` to position `to`
    Move {
        from: SingleOrRange,
        to: u32
    },
    /// Moves the song with id `from` to position `to`
    MoveId {
        from: String,
        to: u32
    },
    /// Finds songs in current playlist with strict matching
    PlaylistFind {
        tag: String,
        needle: String,
    },
    /// Displays either a list of songs, or the song with id if Some
    PlaylistId(Option<u32>),
    /// Displays a list of all songs in playlist, or if position/range is passed,
    /// displays information only for the songs in range/at position
    PlaylistInfo(Option<SingleOrRange>),
    /// Searches case-insensitively for partial matches in the current playlist
    PlaylistSearch {
        tag: TagType,
        needle: String
    },
    /// Displays changes songs currently in the playlist since given version.
    ///
    /// Start and end positions may be given to limit the output to changes in the
    /// given range.
    ///
    /// To detect songs that were deleted at the end of the playlist, use playlistlength returned
    /// by status command (TODO look at this description)
    PlaylistChanges {
        version: String,
        range: Option<Range>
    },
    /// Displays changes songs currently in the playlist since given version.
    ///
    /// This function only return the position and the id of the changed song, and so is more
    /// bandwidth friendly.
    PlaylistChangesPositionId {
        version: String,
        range: Option<Range>
    },
    /// Set the priority of the specified songs.
    ///
    /// Priority alters what order songs will be played in random mode. Songs with
    /// higher priority are played first. The default priority is 0. Max is 255
    Priority {
        priority: u8,
        songs: Vec<SingleOrRange>,
    },
    /// Same as `Priority`, except songs are referenced by id
    PriorityId {
        priority: u8,
        songs: Vec<String>,
    },
    /// Specifies the part of a song in the current playlist that should be played.
    ///
    /// The range specifies the start and end offsets in seconds. No range means play the whole
    /// song. This command will be ignored if the song is currently playing
    RangeId {
        id: String,
        range: Range,
    },
    /// Randomly reorders the playlist between the two ends of the range.
    Shuffle(Range),
    /// Swaps the position of the two songs (given by position in playlist)
    Swap(u32, u32),
    /// Swaps the position of the two songs (given by song id)
    SwapId(String, String),
    /// Adds tag to given song. Only affects song in playlist (not in db). I don't really
    /// understand this
    AddTagId {
        id: String,
        tag: (TagType, String),
    },
    /// Clears a tag on a given song. Only affects song in playlist (not in db). I don't really
    /// understand this
    ClearTagId {
        id: String,
        tag: TagType,
    },
    /// Lists songs in the given playlist.
    ListPlaylist(String),
    /// Lists songs with metadata in the given playlist.
    ListPlaylistInfo(String),
    /// Lists playlists in the playlist directory
    ListPlaylists,
    /// Loads a playlist into the current queue. If a range is supplied, only part of the playlist
    /// will be loaded (matching the range).
    Load {
        name: String,
        range: Option<Range>,
    },
    /// Adds the given song to the playlist `<playlist>.m3u`. If the playlist does not exist, it will
    /// be created.
    PlaylistAdd {
        playlist: String,
        song: String,
    },
    /// Clears the playlist (Will append the `.m3u` suffix).
    PlaylistClear(String),
    /// Deletes song at given position from playlist `<playlist>.m3u`.
    PlaylistDelete {
        playlist: String,
        song: u32,
    },
    /// Moves a song from a position to a new position in the given playlist
    PlaylistMove {
        playlist: String,
        from: u32,
        to: u32,
    },
    /// Renames a playlist
    Rename {
        old_name: String,
        new_name: String,
    },
    /// Removes a playlist
    Remove(String),
    /// Saves the current playlist to the given name.
    Save(String),
    /// Counts the number of songs and their total playtime in the database matching the given tag
    /// exactly.
    ///
    /// The group option can be used to sum over the given tag, rather than to a single value.
    ///
    /// # Examples
    ///
    /// Count all songs and playtime for all Beatles tracks:
    ///
    /// ```ignore
    /// Count {
    ///     tag: (TagType::Artist, "The Beatles".into()),
    ///     group: None,
    /// }
    /// ```
    ///
    /// Count all songs and playtime for rock tracks and sum them up by artist:
    ///
    /// ```ignore
    /// Count {
    ///     tag: (TagType::Genre, "Rock".into()),
    ///     group: TagType::Artist,
    /// }
    /// ```
    Count {
        tag: (TagType, String),
        group: Option<TagType>,
    },
    /// Counts the number of songs and groups them using the given tag type.
    ///
    /// > **Aside**: In the underlying protocol this is a variant of the previous command, but it is
    /// > easier to provide type safety by splitting it out (don't have to introduce a new enum).
    GroupCount(TagType),

}

impl Dispatch for Command {
    fn dispatch(&self, w: &mut io::Write) -> io::Result<()>  {
        use self::Command as Cmd;

        match *self {
            Cmd::ClearError => write!(w, "clearerror\n"),
            Cmd::CurrentSong => write!(w, "currentsong\n"),
            Cmd::Idle(ref sub) => unimplemented!(),
            Cmd::Status => write!(w, "status\n"),
            Cmd::Stats => write!(w, "stats\n"),
            Cmd::Consume(on) => if on {
                write!(w, "consume 1\n")
            } else {
                write!(w, "consume 0\n")
            },
            Cmd::Crossfade(secs) => write!(w, "crossfade {}\n", secs),
            Cmd::MixRampDB(dbs) => write!(w, "mixrampdb {}\n", dbs),
            Cmd::MixRampDelay(amt) => match amt {
                Some(a) => write!(w, "mixrampdelay {}\n", a),
                None => write!(w, "mixrampdelay nan\n"),
            },
            Cmd::Random(on) => if on {
                write!(w, "random 1\n")
            } else {
                write!(w, "random 0\n")
            },
            Cmd::Repeat(on) => if on {
                write!(w, "repeat 1\n")
            } else {
                write!(w, "repeat 0\n")
            },
            Cmd::Volume(vol) => write!(w, "setvol {}\n", vol),
            Cmd::Single(on) => if on {
                write!(w, "single 1\n")
            } else {
                write!(w, "single 0\n")
            },
            Cmd::ReplayGainMode(mode) => match mode {
                ReplayGainMode::Off => write!(w, "replay_gain_mode off\n"),
                ReplayGainMode::Track => write!(w, "replay_gain_mode track\n"),
                ReplayGainMode::Album => write!(w, "replay_gain_mode album\n"),
                ReplayGainMode::Auto => write!(w, "replay_gain_mode auto\n"),
            },
            Cmd::ReplayGainStatus => write!(w, "replay_gain_status\n"),
            Cmd::Next => write!(w, "next\n"),
            Cmd::Pause(on) => if on {
                write!(w, "pause 1\n")
            } else {
                write!(w, "pause 0\n")
            },
            Cmd::Play(pos) => write!(w, "play {}\n", pos),
            Cmd::PlayId(ref id) => write!(w, "playid {}\n", id),
            Cmd::Previous => write!(w, "previous\n"),
            Cmd::Seek {
                song_position: pos,
                time: time
            } => unimplemented!(),
            Cmd::SeekId {
                song_id: ref song_id,
                time: time
            } => unimplemented!(),
            Cmd::SeekCurrent(pos) => unimplemented!(),
            Cmd::Stop => write!(w, "stop\n"),
            Cmd::Add(ref uri) => write!(w, "add {}\n", uri),
            Cmd::AddId {
                uri: ref uri,
                position: ref position
            } => match *position {
                Some(pos) => write!(w, "addid {} {}\n", uri, pos),
                None => write!(w, "addid {}\n", uri),
            },
            Cmd::Clear => write!(w, "clear\n"),
            Cmd::Delete(s_or_r) => write!(w, "delete {}\n", s_or_r),
            Cmd::DeleteId(ref id) => write!(w, "deleteid {}\n", id),
            Cmd::Move {
                from: from,
                to: to
            } => write!(w, "move {} {}\n", from, to),
            Cmd::MoveId {
                from: ref from,
                to: to
            } => write!(w, "moveid {} {}\n", from, to),
            Cmd::PlaylistFind {
                tag: ref tag,
                needle: ref needle,
            } => write!(w, "playlistfind {} {}\n", tag, needle),
            Cmd::PlaylistId(song) => match song {
                Some(song) => write!(w, "playlistid {}\n", song),
                None => write!(w, "playlistid\n"),
            },
            Cmd::PlaylistInfo(s_or_r) => match s_or_r {
                Some(s_or_r) => write!(w, "playlistinfo {}\n", s_or_r),
                None => write!(w, "playlistinfo\n"),
            },
            Cmd::PlaylistSearch {
                tag: ref tag,
                needle: ref needle
            } => write!(w, "playlistfind {} {}\n", tag, needle),
            Cmd::PlaylistChanges {
                version: ref version,
                range: range
            } => match range {
                Some(range) => write!(w, "plchanges {} {}\n", version, range),
                None => write!(w, "plchanges {}\n", version),
            },
            Cmd::PlaylistChangesPositionId {
                version: ref version,
                range: range,
            } => match range {
                Some(range) => write!(w, "plchangesposid {} {}\n", version, range),
                None => write!(w, "plchangesposid {}\n", version),
            },
            Cmd::Priority {
                priority: ref priority,
                songs: ref songs,
            } => {
                write!(w, "prio {}", priority)?;
                for song_group in songs {
                    write!(w, " {}", song_group)?;
                }
                write!(w, "\n")
            },
            Cmd::PriorityId {
                priority: ref priority,
                songs: ref songs,
            } => {
                write!(w, "prioid {}", priority)?;
                for song_group in songs {
                    write!(w, " {}", song_group)?;
                }
                write!(w, "\n")
            },
            Cmd::RangeId {
                id: ref id,
                range: ref range,
            } => write!(w, "rangeid {} {}\n", id, range),
            Cmd::Shuffle(range) => write!(w, "suffle {}\n", range),
            Cmd::Swap(pos1, pos2) => write!(w, "swap {} {}\n", pos1, pos2),
            Cmd::SwapId(ref id1, ref id2) => write!(w, "swapid {} {}\n", id1, id2),
            Cmd::AddTagId {
                id: ref id,
                tag: ref tag
            } => write!(w, "addtagid {} {} {}\n", id, tag.0, tag.1),
            Cmd::ClearTagId {
                id: ref id,
                tag: tag,
            } => write!(w, "cleartagid {} {}\n", id, tag),
            Cmd::ListPlaylist(ref name) => write!(w, "listplaylist {}\n", name),
            Cmd::ListPlaylistInfo(ref name) => write!(w, "listplaylistinfo {}\n", name),
            Cmd::ListPlaylists => write!(w, "listplaylists\n"),
            Cmd::Load {
                name: ref name,
                range: range
            } => match range {
                Some(range) => write!(w, "load {} {}\n", name, range),
                None => write!(w, "load {}\n", name),
            },
            Cmd::PlaylistAdd {
                playlist: ref playlist,
                song: ref song,
            } => write!(w, "playlistadd {} {}\n", playlist, song),
            Cmd::PlaylistClear(ref name) => write!(w, "playlistclear {}\n", name),
            Cmd::PlaylistDelete {
                playlist: ref playlist,
                song: song,
            } => write!(w, "playlistdelete {} {}\n", playlist, song),
            Cmd::PlaylistMove {
                playlist: ref playlist,
                from: from,
                to: to,
            } => write!(w, "playlistmove {} {} {}\n", playlist, from, to),
            Cmd::Rename {
                old_name: ref old_name,
                new_name: ref new_name,
            } => write!(w, "rename {} {}\n", old_name, new_name),
            Cmd::Remove(ref name) => write!(w, "rm {}\n", name),
            Cmd::Save(ref name) => write!(w, "save {}\n", name),
            Cmd::Count {
                tag: ref tag,
                group: group,
            } => match group {
                 Some(group) => write!(w, "count {} {} group {}\n", tag.0, tag.1, group),
                 None => write!(w, "count {} {}\n", tag.0, tag.1),
            },
            Cmd::GroupCount(tag) => write!(w, "count group {}\n", tag),
            /*
            */
            _ => unimplemented!(),
        }
    }
}

impl ParseResponse for Command {
    type ResponseType = CommandResponse;

    fn parse_response<'a>(&self, i: &'a [u8])
        -> IResult<&'a [u8], Self::ResponseType>
    {
        use self::Command::*;
        match *self {
            ClearError => IResult::Done(i, CommandResponse::Blank),
            CurrentSong => IResult::Done(i, CommandResponse::Blank),
            Idle(ref subs) => unimplemented!(),
            Status => parse_status_response(i),
            Stats => parse_stats_response(i),
            Consume(_) => IResult::Done(i, CommandResponse::Blank),
            Crossfade(_) => IResult::Done(i, CommandResponse::Blank),
            MixRampDB(_) => IResult::Done(i, CommandResponse::Blank),
            MixRampDelay(_) => IResult::Done(i, CommandResponse::Blank),
            Random(_) => IResult::Done(i, CommandResponse::Blank),
            Repeat(_) => IResult::Done(i, CommandResponse::Blank),
            Volume(_) => IResult::Done(i, CommandResponse::Blank),
            Single(_) => IResult::Done(i, CommandResponse::Blank),
            ReplayGainMode(_) => IResult::Done(i, CommandResponse::Blank),
            ReplayGainStatus => unimplemented!(),
            Next => IResult::Done(i, CommandResponse::Blank),
            Pause(_) => IResult::Done(i, CommandResponse::Blank),
            Play(_) => IResult::Done(i, CommandResponse::Blank),
            PlayId(_) => IResult::Done(i, CommandResponse::Blank),
            Previous => IResult::Done(i, CommandResponse::Blank),
            Seek {
                song_position: u32,
                time: Duration,
            } => IResult::Done(i, CommandResponse::Blank),
            SeekId {
                song_id: ref String,
                time: Duration,
            } => IResult::Done(i, CommandResponse::Blank),
            SeekCurrent(Duration) => IResult::Done(i, CommandResponse::Blank),
            Stop => IResult::Done(i, CommandResponse::Blank),
            _ => unimplemented!()
        }
        //IResult::Done(i, res)
    }
}

fn parse_single_status_response<'a>(i: &'a[u8], status: &mut MaybeStatus) -> IResult<&'a[u8], ()> {
    alt!(i,
        map_res!(parse_status_line!(b"volume"), parse_bytes::<u8>) => { |o| {
            status.volume = Some(o);
        }}
        | flat_map!(parse_status_line!(b"repeat"), parse_num_bool) => { |o| {
            status.repeat = Some(o);
        }}
        | flat_map!(parse_status_line!(b"random"), parse_num_bool) => { |o| {
            status.random = Some(o);
        }}
        | flat_map!(parse_status_line!(b"single"), parse_num_bool) => { |o| {
            status.single = Some(o);
        }}
        | flat_map!(parse_status_line!(b"consume"), parse_num_bool) => { |o| {
            status.consume = Some(o);
        }}
        | map_res!(parse_status_line!(b"playlist"), parse_bytes::<u32>) => { |o| {
            status.playlist = Some(o);
        }}
        | map_res!(parse_status_line!(b"playlistlength"), parse_bytes::<u32>) => { |o| {
            status.playlist_length = Some(o);
        }}
        | map_res!(parse_status_line!(b"mixrampdb"), parse_bytes::<f32>) => { |o| {
            status.mix_ramp_db = Some(o);
        }}
        | flat_map!(parse_status_line!(b"state"), parse_status_state) => { |o| {
            status.state = Some(o);
        }}
        | map_res!(parse_status_line!(b"xfade"), parse_bytes::<u32>) => { |o| {
            status.crossfade = Some(o);
        }}
        | map_res!(parse_status_line!(b"song"), parse_bytes::<u32>) => { |o| {
            status.song = Some(o);
        }}
        | map_res!(parse_status_line!(b"songid"), parse_bytes::<u32>) => { |o| {
            status.song_id = Some(o);
        }}
        | parse_status_line!(b"time") => { |_| () } // ignored
        | flat_map!(parse_status_line!(b"elapsed"), parse_time) => { |o| {
            status.elapsed = Some(o);
        }}
        | map_res!(parse_status_line!(b"bitrate"), parse_bytes::<u32>) => { |o| {
            status.bitrate = Some(o);
        }}
        | flat_map!(parse_status_line!(b"duration"), parse_time) => { |o| {
            status.duration = Some(o);
        }}
        | flat_map!(parse_status_line!(b"audio"), parse_audio) => { |o| {
            status.audio = Some(o);
        }}
        | map_res!(parse_status_line!(b"nextsong"), parse_bytes::<u32>) => { |o| {
            status.next_song = Some(o);
        }}
        | map_res!(parse_status_line!(b"nextsongid"), parse_bytes::<u32>) => { |o| {
            status.next_song_id = Some(o);
        }}
    )
}

fn parse_status_response(i: &[u8]) -> IResult<&[u8], CommandResponse> {
    let mut status: MaybeStatus = Default::default();
    //trace_macros!(true);
    //trace_macros!(false);
    let mut i_inner = i;

    loop {
        match parse_single_status_response(i_inner, &mut status) {
            IResult::Done(i, _) => { i_inner = i; }
            IResult::Error(e) => { break; }
            IResult::Incomplete(n) => { return IResult::Incomplete(n); }
        }
    }
    match status.try_into() {
        Some(s) => IResult::Done(i_inner, CommandResponse::Status(s)),
        None => IResult::Error(error_position!(ErrorKind::Custom(0), i_inner))
    }
}

#[test]
fn test_parse_status_response() {
    let input = b"volume: 80
repeat: 1
random: 1
single: 0
consume: 0
playlist: 4
playlistlength: 1
mixrampdb: 0.000000
state: play
xfade: 1000000000
song: 0
songid: 9
time: 80:302
elapsed: 80.074
bitrate: 320
audio: 44100:24:2
nextsong: 0
nextsongid: 9
list_OK
";
    assert_eq!(
        parse_status_response(&input[..]),
        IResult::Done(&b"list_OK\n"[..], CommandResponse::Status(Status {
            volume: 80,
            repeat: true,
            random: true,
            single: false,
            consume: false,
            playlist: 4,
            playlist_length: 1,
            mix_ramp_db: 0.0,
            state: State::Play,
            crossfade: 1_000_000_000,
            song: 0,
            song_id: 9,
            elapsed: Duration::new(80, 74_000_000),
            duration: None,
            bitrate: 320,
            audio: (44100, 24, 2),
            next_song: 0,
            next_song_id: 9,
            updating_db: None,
            error: None
        }))
    );
}

fn parse_single_stats_response<'a>(i: &'a[u8], stats: &mut MaybeStats) -> IResult<&'a[u8], ()> {
    alt!(i,
        map_res!(parse_status_line!(b"artists"), parse_bytes::<u64>) => { |o| {
            stats.artists = Some(o);
        }}
        | map_res!(parse_status_line!(b"albums"), parse_bytes::<u64>) => { |o| {
            stats.albums = Some(o);
        }}
        | map_res!(parse_status_line!(b"songs"), parse_bytes::<u64>) => { |o| {
            stats.songs = Some(o);
        }}
        | map_res!(parse_status_line!(b"uptime"), parse_bytes::<u64>) => { |o| {
            stats.uptime = Some(Duration::from_secs(o));
        }}
        | map_res!(parse_status_line!(b"db_playtime"), parse_bytes::<u64>) => { |o| {
            stats.db_playtime = Some(Duration::from_secs(o));
        }}
        | map_res!(parse_status_line!(b"db_update"), parse_bytes::<i64>) => { |o| {

            stats.db_update = Some(UTC.timestamp(o, 0));
        }}
        | map_res!(parse_status_line!(b"playtime"), parse_bytes::<u64>) => { |o| {
            stats.playtime = Some(Duration::from_secs(o));
        }}
    )
}

fn parse_stats_response(i: &[u8]) -> IResult<&[u8], CommandResponse> {
    let mut stats: MaybeStats = Default::default();
    //trace_macros!(true);
    //trace_macros!(false);
    let mut i_inner = i;

    loop {
        match parse_single_stats_response(i_inner, &mut stats) {
            IResult::Done(i, _) => { i_inner = i; }
            IResult::Error(e) => { break; }
            IResult::Incomplete(n) => { return IResult::Incomplete(n); }
        }
    }
    match stats.try_into() {
        Some(s) => IResult::Done(i_inner, CommandResponse::Stats(s)),
        None => IResult::Error(error_position!(ErrorKind::Custom(0), i_inner))
    }
}

named!(parse_status_state<State>,
    alt!(
        map!(tag!("play"), |_| State::Play) |
        map!(tag!("pause"), |_| State::Pause) |
        map!(tag!("stop"), |_| State::Stop)
    )
);

#[test]
fn test_parse_status_state() {
    let input = b"play";
    assert_eq!(
        parse_status_state(&input[..]),
        IResult::Done(&b""[..], State::Play)
    );
}

named!(parse_audio<(u32, u32, u32)>,
    do_parse!(
        sample_rate: map_res!(digit, |i| parse_bytes::<u32>(i)) >>
        tag!(":") >>
        bit_depth: map_res!(digit, |i| parse_bytes::<u32>(i)) >>
        tag!(":") >>
        channels: map_res!(digit, |i| parse_bytes::<u32>(i)) >>
        ((sample_rate, bit_depth, channels))
    )
);

#[test]
fn test_parse_audio() {
    let input = b"44100:24:2";
    assert_eq!(
        parse_audio(&input[..]),
        IResult::Done(&b""[..], (44100, 24, 2))
    );
}

named!(parse_time<Duration>,
    do_parse!(
        secs: map_res!(digit, |i| parse_bytes::<u64>(i)) >>
        tag!(b".") >>
        nanos: map_res!(digit, |i| parse_bytes::<u32>(i).map(
            |val| val * (1_000_000_000 / 10u32.pow(i.len() as u32))
        )) >>
        (Duration::new(secs, nanos))
    )
);

#[derive(Clone, Debug, PartialEq)]
pub enum CommandResponse {
    Blank,
    Tmp,
    Status(Status),
    Stats(Stats),
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use protocol::Dispatch;

    #[test]
    fn command_list_dispatch() {
        let mut s_raw: Vec<u8> = Vec::new();
        let cmd_list = CommandList::new();
        cmd_list.dispatch(&mut s_raw).unwrap();
        assert_eq!(
            str::from_utf8(&s_raw[..]).unwrap(),
            "command_list_ok_begin\ncommand_list_end\n"
        )
    }
}
