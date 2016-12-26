# musicpd

## Server spec (because ones I've read are ambiguous)

### State
 - Filesystem
 - Playlists
 - Play info
   - Current playlist
   - Position (uint)
   - Paused (bool)
   - repeat (bool)
   - single (bool)
 - *are there more?*

### Grammar

#### Client - Server
All messages are utf8, although all commands are ascii (variables can be
non-ascii). Floats don't allow exponential syntax `2.14e21`

`command_sequence` is the starting point

```
command_sequence = (command "\n")+

command = "clearerror"
        | "currentsong"
        | "idle " subsystems*
        | "status"
        | "stats"
        | "consume " num_bool
        | "crossfade " uint
        | "mixrampdb -" uint # actual value is -value, so 17 is -17dB
        | "mixrampdelay " uint
        | "random " num_bool
        | "repeat " num_bool
        | "setvol " uint # capped at 100
        | "single " num_bool
        | "replay_gain_mode " replay_gain_mode
        | "replay_gain_status"
        | "volume " uint # (deprecated) relative volume change
        | "next"
        | "pause " num_bool
        | "play " uint
        | "playid " uint
        | "previous"
        | "seek " uint " " positive_float
        | "seekid " uint " " positive_float
        | "seekcur " positive_float
        | "stop"
        # TODO
num_bool = "0" | "1" # represents boolean value, 1 = true/on, false = false/off

replay_gain_mode = "off"
                 | "track"
                 | "album"
                 | "auto"
```
