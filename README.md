# Pomelo 0.2.0
 
Pomelo is a desktop media player with Youtube search, playback, and download functionality.

## Current features
- Play videos locally from the computer, with shuffle and reverse playback.

- Search for videos, channels, or playlists from Youtube.

- Play videos or playlists straight from Youtube.

- Download videos or playlists from Youtube.
 
## Known issues
- Searching and downloading might stop working, usually due to Youtube API changes. If this happens, try changing the Invidious and/or Yt-dlp settings.

- Downloads may be separated into 2 files for audio and video. This can usually be fixed by enabling the nightly build of yt-dlp.

## Installation

Before installing Pomelo, you will need to install one or both of the following dependencies:

### Gstreamer ( Required )
Check [the official website](https://gstreamer.freedesktop.org/download) for instructions on how to install.

If you're installing gstreamer on Windows, you will also need to add the path to the gstreamer binaries ( example: C:\gstreamer\1.0\msvc_x86_64\bin ) to the [PATH environment variable.](https://www.computerhope.com/issues/ch000549.htm)

### FFmpeg ( Optional, but required for most download features )

#### Linux
On some distros ( Ubuntu, Debian, Arch, etc. ) you can install FFmpeg using a package manager. Otherwise, you can download FFmpeg from the [git repository](https://github.com/FFmpeg/FFmpeg/tree/master), then follow the instructions [here](https://github.com/FFmpeg/FFmpeg/blob/master/INSTALL.md) to install.

#### Windows
1. [Download the executables for Windows under "More downloading options"](https://www.ffmpeg.org/download.html)
    - You can download either the "full" or "essential" build, they both should work.
2. Create an "ffmpeg" folder somewhere on your computer ( example: C:\ffmpeg ), then extract the contents of the downloaded zip file and place them in this folder.
3. Add the path to the ffmpeg executables ( example: C:\ffmpeg\ffmpeg-7.0.2-full_build\bin ) to the [PATH environment variable.](https://www.computerhope.com/issues/ch000549.htm)

After installind the dependencies, download the latest release [here](https://github.com/sudfud/Pomelo/releases), then simply extract the executable and place it wherever you want.

## Compatibility
Pomelo should work on most modern Windows and Linux systems, though testing on this has been limited. It may also be buildable on MacOS, but this is untested. Below is a list of systems that are confirmed to work.
If you have Pomelo working on a system not listed here, please let me know so it can be added to the list.

### Windows
- Windows 11, 10

### Linux
- Ubuntu
