# Pomelo 0.1.0
 
Pomelo is a desktop media player with Youtube search, playback, and download functionality.

## Current features
- Play videos locally from the computer, with shuffle and reverse playback.

- Search for videos, channels, or playlists from Youtube.

- Play videos or playlists straight from Youtube.

- Download videos or playlists from Youtube.
 
## Known issues
- Searching and downloading might stop working, usually due to Youtube API changes. If this happens, try changing the Invidious and/or Yt-dlp settings.

- Downloads may be separated into 2 files for audio and video. This can usually be fixed by enabling the nightly build of yt-dlp.

- Downloads may be in different formats (mp4 or webm) or qualities. This presumably depends on if ffmpeg is installed. Format/Quality options will be added as development progresses.

## Installation

### Gstreamer
To run Pomelo, you'll need to have gstreamer installed on your computer. Check [the official website](https://gstreamer.freedesktop.org/download) for instructions on how to install.

If you're installing gstreamer on Windows, you will also need to add the path to the gstreamer binaries ( example: C:\gstreamer\1.0\msvc_x86_64\bin ) to the [PATH environment variable.](https://www.computerhope.com/issues/ch000549.htm)

After installing gstreamer, you should be able to simply extract and run Pomelo.

It is recommended to keep Pomelo in its own separate folder, as it will create files and folders in the same location as the program.

## Compatibility
Pomelo should work on most modern Windows and Linux systems, though testing on this has been limited. It may also be buildable on MacOS, but this is untested. Below is a list of systems that are confirmed to work.
If you have Pomelo working on a system not listed here, please let me know so it can be added to the list.

### Windows
- Windows 11, 10

### Linux
- Ubuntu
