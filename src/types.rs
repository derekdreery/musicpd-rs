
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
pub struct Tag {
    /// The value of the tag.
    pub value: String,
    /// The tag type.
    pub tag_type: TagType
}

/// The following tags are supported by MPD
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


