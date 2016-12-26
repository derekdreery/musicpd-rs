use std::io;

use super::Dispatch;

#[derive(Clone, Debug, PartialEq)]
pub struct CommandList(Vec<Command>);

impl CommandList {
    /// Create an empty command list
    pub fn new() -> Self {
        CommandList(Vec::new())
    }
}

impl Dispatch for CommandList {
    /// Dispatch the command list to the server
    fn dispatch(&self, w: &mut io::Write) -> io::Result<()> {
        w.write_all(b"command_list_begin\n")?;
        for cmd in &self.0 {
            println!("{:?}", cmd);
        }
        w.write_all(b"command_list_end\n")?;
        Ok(())
    }
}

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
        time: f32
    },
    /// Plays from the given time in the song with the given id
    SeekId {
        song_id: String,
        time: f32
    },
    /// Plays from the given time in the current song.
    SeekCurrent(f32),
    /// Stop playing
    Stop,
}

impl Dispatch for Command {
    fn dispatch(&self, w: &mut io::Write) -> io::Result<()>  {
        w.write_all(match self {
            _ => unimplemented!()
        })?;
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReplayGainMode {
    Off,
    Track,
    Album,
    Auto
}

/// Some commands require a range (e.g. delete)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Range {
    /// The start of the range
    pub start: usize,
    /// (optional) The end of the range
    ///
    /// If `None`, the maximum possible range is assumed
    pub end: Option<usize>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TagType {
    Artist,
    ArtistSort,
    Album,
    AlbumSort,
    AlbumArtist,
    AlbumArtistSort,
    Title,
    Track,
    Name,
    Genre,
    Date,
    Composer,
    Performer,
    Comment,
    Disc,
    MusicbrainzArtistId,
    MusicbrainzAlbumId,
    MusicbrainzAlbumArtistId,
    MusicbrainzTrackId,
    MusicbrainzReleaseTrackId
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
            "command_list_begin\ncommand_list_end\n"
        )
    }
}
