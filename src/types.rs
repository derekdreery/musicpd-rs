use std::time::Duration;
use chrono::{DateTime, UTC, TimeZone};
use std::default;
use std::fmt;

/// The possible error types sent from mpd
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CmdErrorType {
    NotList,
    Arg,
    Password,
    Permission,
    Unknown,

    NoExist,
    PlaylistMax,
    System,
    PlaylistLoad,
    UpdateAlready,
    PlayerSync,
    Exist
}

impl CmdErrorType {
    /// Maps codes to error types
    pub fn from_code(code: &[u8]) -> Option<CmdErrorType> {
        use self::CmdErrorType::*;
        match code {
            b"1" => Some(NotList),
            b"2" => Some(Arg),
            b"3" => Some(Password),
            b"4" => Some(Permission),
            b"5" => Some(Unknown),
            b"50" => Some(NoExist),
            b"51" => Some(PlaylistMax),
            b"52" => Some(System),
            b"53" => Some(PlaylistLoad),
            b"54" => Some(UpdateAlready),
            b"55" => Some(PlayerSync),
            b"56" => Some(Exist),
            _ => None
        }
    }
}

/// The error returned from the server for failed commands
#[derive(Debug, Clone, PartialEq)]
pub struct CmdError {
    /// The error type
    pub error_type: CmdErrorType,
    /// The index of the command that caused the error
    pub command_no: usize,
    /// The name of the command that caused the error
    pub command_name: String,
    /// Any (hopefully) helpful message text from the server
    pub message_text: String
}

/// A piece of textual information about a track of music or sound.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    /// The value of the tag.
    pub value: String,
    /// The tag type.
    pub tag_type: TagType
}

/// The following tags are supported by MPD
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TagType {
    /// The artist name. Its meaning is not well-defined; see composer and performer for more
    /// specific tags.
    Artist,
    /// Same as artist, but for sorting. This usually omits prefixes such as "The".
    ArtistSort,
    /// The album name.
    Album,
    /// Same as album, but for sorting.
    AlbumSort,
    /// On multi-artist albums, this is the artist name which shall be used for the whole album.
    /// The exact meaning of this tag is not well-defined.
    AlbumArtist,
    /// Same as albumartist, but for sorting.
    AlbumArtistSort,
    /// The song title.
    Title,
    /// The track number within the album.
    Track,
    /// A name for this song. This is not the song title. The exact meaning of this tag is not
    /// well-defined. It is often used by badly configured internet radio stations with broken tags
    /// to squeeze both the artist name and the song title in one tag.
    Name,
    /// The music genre.
    Genre,
    /// The song's release date. This is usually a 4-digit year.
    Date,
    /// The artist who composed the song.
    Composer,
    /// The artist who performed the song.
    Performer,
    /// A human-readable comment about this song. The exact meaning of this tag is not
    /// well-defined.
    Comment,
    /// The disc number in a multi-disc album.
    Disc,
    /// The artist id in the MusicBrainz database.
    MusicbrainzArtistId,
    /// The album id in the MusicBrainz database.
    MusicbrainzAlbumId,
    /// The album artist id in the MusicBrainz database.
    MusicbrainzAlbumArtistId,
    /// The track id in the MusicBrainz database.
    MusicbrainzTrackId,
    /// The release track id in the MusicBrainz database.
    MusicbrainzReleaseTrackId
}

impl fmt::Display for TagType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Artist => write!(f, "artist"),
            ArtistSort => write!(f, "artistsort"),
            Album => write!(f, "album"),
            AlbumSort => write!(f, "albumsort"),
            AlbumArtist => write!(f, "albumartist"),
            AlbumArtistSort => write!(f, "albumartistsort"),
            Title => write!(f, "title"),
            Track => write!(f, "track"),
            Name => write!(f, "name"),
            Genre => write!(f, "genre"),
            Date => write!(f, "date"),
            Composer => write!(f, "composer"),
            Performer => write!(f, "performer"),
            Comment => write!(f, "comment"),
            Disc => write!(f, "disc"),
            MusicbrainzArtistId => write!(f, "musicbrainz_artistid"),
            MusicbrainzAlbumId => write!(f, "musicbrainz_albumid"),
            MusicbrainzAlbumArtistId => write!(f, "musicbrainz_albumartistid"),
            MusicbrainzTrackId => write!(f, "musicbrainz_trackid"),
            MusicbrainzReleaseTrackId => write!(f, "musicbrainz_releasetrackid"),
        }
    }
}

/// The types of subsystem that can be subscribed to by `Command::Idle`
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SubSystem {
    Database,
    Update,
    StoredPlaylist,
    Playlist,
    Player,
    Mixer,
    Output,
    Options,
    Sticker,
    Subscription,
    Message
}

/// Some commands require a range (e.g. delete)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Range {
    /// The start of the range
    pub start: u32,
    /// (optional) The end of the range
    ///
    /// If `None`, the maximum possible range is assumed
    pub end: Option<u32>,
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(end) = self.end {
            write!(f, "{}:{}", self.start, end)
        } else {
            write!(f, "{}:", self.start)
        }
    }
}

/// Either a single value or a range of values.
///
/// Used as parameter for certain commands
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SingleOrRange {
    /// A single value
    Single(u32),
    /// A range of values
    Range(Range),
}

impl From<u32> for SingleOrRange {
    fn from(val: u32) -> SingleOrRange {
        SingleOrRange::Single(val)
    }
}

impl From<Range> for SingleOrRange {
    fn from(val: Range) -> SingleOrRange {
        SingleOrRange::Range(val)
    }
}

impl fmt::Display for SingleOrRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SingleOrRange::Single(val) => write!(f, "{}", val),
            SingleOrRange::Range(r) => write!(f, "{}", r),
        }
    }
}

/// Information about what mpd is doing.
///
/// This is returned from the `Status` command
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    /// The current volume
    pub volume: u8,
    /// Whether repeat mode is on
    pub repeat: bool,
    /// Whether random mode is on
    pub random: bool,
    /// Whether single mode is on (dunno what this means)
    pub single: bool,
    /// Whether songs should be removed from the playlist as they are played
    pub consume: bool,
    /// The playlist version number
    pub playlist: u32,
    /// The number of songs in the playlist
    pub playlist_length: u32,
    /// Whether mpd is playing, paused, or stopped
    pub state: State,
    /// The position in the playlist of the currently playing song
    pub song: u32,
    /// The song id of the currently playing song
    pub song_id: u32,
    /// The playlist position of the next song to play
    pub next_song: u32,
    /// The song id of the next song to play
    pub next_song_id: u32,
    /// How far through the current song mpd is
    pub elapsed: Duration,
    /// The length of the current song
    pub duration: Option<Duration>,
    /// The bitrate at the current position of the current song in kbps
    pub bitrate: u32,
    /// The crossfade time in seconds
    pub crossfade: u32, // may need more
    /// The length of the mixramp time in seconds
    pub mix_ramp_db: f32,
    /// Audio information: (sample rate, bits, channels)
    pub audio: (u32, u32, u32), // check types
    /// The job id (TODO needs more info)
    pub updating_db: Option<u32>,
    /// If there is an error that hasn't been cleared, it will be here
    pub error: Option<String>
}

#[derive(Clone, Debug, PartialEq)]
/// Helper struct to build status from responses
pub struct MaybeStatus {
    pub volume: Option<u8>,
    pub repeat: Option<bool>,
    pub random: Option<bool>,
    pub single: Option<bool>,
    pub consume: Option<bool>,
    pub playlist: Option<u32>,
    pub playlist_length: Option<u32>,
    pub state: Option<State>,
    pub song: Option<u32>,
    pub song_id: Option<u32>,
    pub next_song: Option<u32>,
    pub next_song_id: Option<u32>,
    pub elapsed: Option<Duration>,
    pub duration: Option<Duration>,
    pub bitrate: Option<u32>,
    pub crossfade: Option<u32>, // may need more
    pub mix_ramp_db: Option<f32>,
    /// (sample rate, bits, channels)
    pub audio: Option<(u32, u32, u32)>, // check types
    pub updating_db: Option<u32>,
    pub error: Option<String>
}

impl default::Default for MaybeStatus {
    fn default() -> Self {
        MaybeStatus {
            volume: None,
            repeat: None,
            random: None,
            single: None,
            consume: None,
            playlist: None,
            playlist_length: None,
            state: None,
            song: None,
            song_id: None,
            next_song: None,
            next_song_id: None,
            elapsed: None,
            duration: None,
            bitrate: None,
            crossfade: None,
            mix_ramp_db: None,
            audio: None,
            updating_db: None,
            error: None,
        }
    }
}

impl MaybeStatus {
    /// Convert into a status if possible, if not return None
    pub fn try_into(&self) -> Option<Status> {
        Some(Status {
            volume: try_opt!(self.volume),
            repeat: try_opt!(self.repeat),
            random: try_opt!(self.random),
            single: try_opt!(self.single),
            consume: try_opt!(self.consume),
            playlist: try_opt!(self.playlist),
            playlist_length: try_opt!(self.playlist_length),
            state: try_opt!(self.state),
            song: try_opt!(self.song),
            song_id: try_opt!(self.song_id),
            next_song: try_opt!(self.next_song),
            next_song_id: try_opt!(self.next_song_id),
            elapsed: try_opt!(self.elapsed),
            duration: self.duration,
            bitrate: try_opt!(self.bitrate),
            crossfade: try_opt!(self.crossfade),
            mix_ramp_db: try_opt!(self.mix_ramp_db),
            audio: try_opt!(self.audio),
            updating_db: self.updating_db,
            error: self.error.clone(),
        })
    }
}

/// Stats about the database
#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    /// Number of artists
    pub artists: u64,
    /// Number of albums
    pub albums: u64,
    /// Number of songs
    pub songs: u64,
    /// Daemon uptime
    pub uptime: Duration,
    /// Sum of durations of all songs
    pub db_playtime: Duration,
    /// Last DB Update
    pub db_update: DateTime<UTC>,
    /// Time length of music played
    pub playtime: Duration,
}

#[derive(Clone, Debug, PartialEq)]
/// Helper struct to build stats from responses
pub struct MaybeStats {
    pub artists: Option<u64>,
    pub albums: Option<u64>,
    pub songs: Option<u64>,
    pub uptime: Option<Duration>,
    pub db_playtime: Option<Duration>,
    pub db_update: Option<DateTime<UTC>>,
    pub playtime: Option<Duration>,
}

impl default::Default for MaybeStats {
    fn default() -> Self {
        MaybeStats {
            artists: None,
            albums: None,
            songs: None,
            uptime: None,
            db_playtime: None,
            db_update: None,
            playtime: None,
        }
    }
}

impl MaybeStats {
    /// Convert into a stats if possible, if not return None
    pub fn try_into(&self) -> Option<Stats> {
        Some(Stats {
            artists: try_opt!(self.artists),
            albums: try_opt!(self.albums),
            songs: try_opt!(self.songs),
            uptime: try_opt!(self.uptime),
            db_playtime: try_opt!(self.db_playtime),
            db_update: try_opt!(self.db_update),
            playtime: try_opt!(self.playtime),
        })
    }
}

/// The current playback state of mpd
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    /// Playing
    Play,
    /// Paused
    Pause,
    /// Stopped
    Stop
}

/// The replay gain mode (TODO what is this?)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReplayGainMode {
    Off,
    Track,
    Album,
    Auto
}
