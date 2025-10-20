# Music Downloader (mdl)

This is a simple music downloader, which downloads and tags music. It acomplishes this using user provided specfiles.

> How does it work?

`mdl` is based on [`yt-dlp`](https://github.com/yt-dlp/yt-dlp), more accurately on its Python API. But wait.. a Rust application 
based on a Python API? Yes, to interface with the `yt-dlp`'s API, the Python bindings provided by 
[`pyo3`](https://crates.io/crates/pyo3) are used.

> Isn't it too complicated?

Yes, a little bit, but I personally like Rust and at the same time don't like Python.

> How to download something?

The idea is as follows:
1. Form the specfile by fetching a URL
2. Edit the specfile - add missing tracks, fill in album information, artists, etc.
3. Download the album
4. Tag the downloaded files

Steps 1, 3 and 4 are done using the CLI, as for step 2 - the specfile is yours to edit. For more detailed instructions see [Usage](#usage).

# Dependencies

- `yt-dlp` preferrably the newest version. This can be easily done with Python's virtual environments.
- Satisfy the Python dependencies for [`pyo3`](https://github.com/PyO3/pyo3).
- (OPTIONAL) `bgutils-ytdl-pot-provider` or another PO Token provider for `yt-dlp`. This is highly recommended to avoid download issues.

# Supported websites

The idea for this tool (for now) is to only work with full albums, not single tracks, although smart users 
may be able to edit a specfile in a way to produce a signle track.

Supported album types:
1. Playlist - playlists containing tracks as their entries
   - YouTube
   - Soundcloud
2. Chapters - videos split into chapters, each being a separate track
   - YouTube only

In the second case it is also possible to fetch many chapter albums at once. If a playlist has many videos, each containing 
chapters, using recursive fetch produces multiple specfiles - each specfile corresponding to a video. This is a fetch only 
convenience, as after obtaining the specfiles, they act as separate albums.

## YouTube URL formats

For playlists and recursive chapters:
- `https://www.youtube.com/playlist?list=<id>`
- `https://www.youtube.com/watch?v=<vid>&list=<id>`

For chapters:
- `https://www.youtube.com/watch?v=<vid>`
- `https://www.youtube.com/watch?v=<vid>&list=<id>`

## Soundcloud URL formats

For playlists:
- `https://soundcloud.com/<author>/sets/<id>`

# Specfile format

For how to edit or write specfiles see the `specdoc` directory. Inside you will find:
- `chapters.toml`: Documentation for chapter album specfiles
- `sc-playlist.toml`: Documentation for playlist album specfiles - what you might expect when fetching from Soundcloud
- `yt-playlist.toml`: Documentation for playlist album specfiles - what you might expect when fetching from YouTube

`sc-playlist.toml` and `yt-playlist.toml` are almost the same, but both are included because Soundcloud offers some more 
metadata than YouTube.

# Usage

1. Fetch a specfile

Find something to download and run `mdl fetch` with the options you want, here's a brief of those options:
```
Fetch a specfile from a URL

Usage: mdl fetch [OPTIONS] <--chapters|--playlist> <URL>...

Arguments:
  <URL>...  URLs to fetch specfiles from

Options:
  -c, --chapters      Fetch a single video's chapters as an album
  -p, --playlist      Fetch a playlist as an album
  -r, --recursive     If using --chapters, fetch all videos in the playlist as if they were separate URLs passed with --chapters
  -o, --outdir <DIR>  Output directory for the specfiles
  -i, --id <ID>       Overrides the specfile's ID
```

Use `mdl help fetch` or `mdl fetch --help` for a more detailed explanation of each option. If you want you may specify multiple 
URLs to fetch multiple specfiles.

2. Edit the specfile

Once the specfile is fetched, it's yours to edit. See [Specfile Format](#specfile-format).

3. Download the media

Run `mdl download` with your specfile (or specfiles). You may customise the behaviour with some options:
```
Download files according to a specfile

Usage: mdl download [OPTIONS] <SPEC>...

Arguments:
  <SPEC>...  Specfiles of albums to download

Options:
  -o, --outdir <DIR>  Output directory for the downloaded files
```

4. Tag the downloaded files

Once downloaded you may use the files as they are, but it is not recommended. Run `mdl tag` to convert them to the format of your 
choice and fill in the metadata from the specfile. The options for `tag` are:
```
Postprocess, split and tag downloaded files

Usage: mdl tag [OPTIONS] --target <FORMAT> <SPEC>...

Arguments:
  <SPEC>...  Specfiles of albums to tag

Options:
  -t, --target <FORMAT>  Audio codec to use [possible values: flac, mp3]
  -i, --indir <DIR>      Directory of the previously downloaded files
  -o, --outdir <DIR>     Output directory for the tagged files
  -k, --keep             Keep the untagged, unprocessed, downloaded files
```
