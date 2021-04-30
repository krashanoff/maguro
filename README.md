# maguro 🐟

A fast YouTube downloader.

## Using the CLI

```
$ # Check which streams are available (in no particular order).
$ maguro -F VfWgE7D1pYY
Displaying available formats for video ID VfWgE7D1pYY:
itag: 018       Quality: medium Mime Type: video/mp4; codecs="avc1.42001E, mp4a.40.2"
itag: 022       Quality: hd720  Mime Type: video/mp4; codecs="avc1.64001F, mp4a.40.2"
itag: 137       Quality: hd1080 Mime Type: video/mp4; codecs="avc1.640028"
itag: 248       Quality: hd1080 Mime Type: video/webm; codecs="vp9"
itag: 136       Quality: hd720  Mime Type: video/mp4; codecs="avc1.4d401f"
itag: 247       Quality: hd720  Mime Type: video/webm; codecs="vp9"
itag: 135       Quality: large  Mime Type: video/mp4; codecs="avc1.4d401f"
itag: 244       Quality: large  Mime Type: video/webm; codecs="vp9"
itag: 134       Quality: medium Mime Type: video/mp4; codecs="avc1.4d401e"
itag: 243       Quality: medium Mime Type: video/webm; codecs="vp9"
itag: 133       Quality: small  Mime Type: video/mp4; codecs="avc1.4d4015"
itag: 242       Quality: small  Mime Type: video/webm; codecs="vp9"
itag: 160       Quality: tiny   Mime Type: video/mp4; codecs="avc1.4d400c"
itag: 278       Quality: tiny   Mime Type: video/webm; codecs="vp9"
itag: 140       Quality: tiny   Mime Type: audio/mp4; codecs="mp4a.40.2"
itag: 251       Quality: tiny   Mime Type: audio/webm; codecs="opus"

$ # Choose a stream to download, and specify the output.
$ maguro -o mp4 -f 133 VfWgE7D1pYY
Starting download of VfWgE7D1pYY...
Completed download of video VfWgE7D1pYY.

$ # The more -v's, the more verbose your output.
$ maguro -vvv -o mp4 VfWgE7D1pYY
```

## Using the Library

maguro also exposes a library for use in other applications. It is fully-asynchronous, and is (hopefully)
intuitive and easy to use. Examples are available in the [examples folder](./examples).

## Disclaimer

This software is created with the purpose of downloading videos with express
permission from their creator(s). The creator of this software is not
responsible for whether this software is used to download copyrighted YouTube
videos.
