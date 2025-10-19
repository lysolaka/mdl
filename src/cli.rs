use std::path::PathBuf;

use anstyle::{AnsiColor, Style};
use log::LevelFilter;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
#[command(propagate_version = true)]
#[command(styles = STYLING)]
pub struct Cli {
    #[command(flatten)]
    pub verbosity: Verbosity,
    #[command(subcommand)]
    pub action: Subcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    Fetch(FetchArgs),
    Download(DownloadArgs),
    Tag(TagArgs),
}

/// Fetch a specfile from a URL
#[derive(Debug, clap::Args)]
pub struct FetchArgs {
    #[command(flatten)]
    pub mode: FetchMode,
    /// If using --chapters, fetch all videos in the playlist as if they were separate URLs passed with --chapters.
    ///
    /// If --id was used for the URL, the specfiles will be named: '<id>_<playlist_index>'. Without a custom ID, the 
    /// specfiles will be named as if they were separate URLs.
    #[arg(short, long, display_order = 2)]
    pub recursive: bool,
    /// Output directory for the specfiles.
    ///
    /// Specfiles will be put to: '<outdir>/<id>.toml'. If this option is missing, specfiles will be saved to the current directory.
    #[arg(short, long, value_name = "DIR", display_order = 3)]
    pub outdir: Option<PathBuf>,
    /// Overrides the specfile's ID. 
    ///
    /// If passed multiple times, the subsequent IDs will be used for subsequent URLs. 
    /// Once exhausted the specfiles will fall back to automatic IDs.
    #[arg(short, long, default_values_t = Vec::<String>::new(), value_name = "ID", display_order = 4)]
    pub id: Vec<String>,
    /// URLs to fetch specfiles from. 
    ///
    /// When passing multiple URLs, make sure that they can be used in the selected mode (either --playlist or --chapters) 
    /// - the URLs cannot be mixed.
    #[arg(value_name = "URL", required = true)]
    pub url: Vec<url::Url>,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct FetchMode {
    /// Fetch a single video's chapters as an album.
    ///
    /// Only YouTube videos are supported. If a URL refers to a video and a playlist, it will be converted to a video URL.
    #[arg(short, long, display_order = 2)]
    pub chapters: bool,
    /// Fetch a playlist as an album.
    ///
    /// Soundcloud and YouTube playlists are supported. For supported URL formats see the documentation.
    #[arg(short, long, display_order = 2)]
    pub playlist: bool,
}

#[derive(Debug, clap::Args)]
pub struct DownloadArgs {
    /// Output directory for the downloaded files.
    ///
    /// The files themselves will be located at '<outdir>/<spec_id>/'. If this option is missing the downloaded files 
    /// will go into './<spec_id>/'.
    #[arg(short, long, value_name = "DIR", display_order = 2)]
    pub outdir: Option<PathBuf>,
    /// Specfiles of albums to download.
    #[arg(value_name = "SPEC", required = true)]
    pub spec: Vec<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct TagArgs {
    /// Audio codec to use.
    ///
    /// For 'mp3' the files will be tagged with ID3v2.3, for 'flac' - Vorbis Comments.
    #[arg(short, long, value_name = "FORMAT", display_order = 2)]
    pub target: crate::specfile::Format,
    /// Directory of the previously downloaded files.
    ///
    /// The files will be looked for in '<indir>/<spec_id>/'. If this option is missing, the downloaded files will be 
    /// looked for in './<spec_id>/'.
    #[arg(short, long, value_name = "DIR", display_order = 3)]
    pub indir: Option<PathBuf>,
    /// Output directory for the tagged files.
    ///
    /// The files themselves will be located at '<outdir>/<album_title>/'. If this option is missing the downloaded 
    /// files will go into './<album_title>/'.
    #[arg(short, long, value_name = "DIR", display_order = 4)]
    pub outdir: Option<PathBuf>,
    /// Keep the untagged, unprocessed, downloaded files.
    ///
    /// Tracks supplied locally (via a 'file://' URL) will not be removed regardless of this option.
    #[arg(short, long, display_order = 5)]
    pub keep: bool,
    /// Specfiles of albums to tag.
    #[arg(value_name = "SPEC", required = true)]
    pub spec: Vec<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct Verbosity {
    /// Increase logging verbosity.
    #[arg(
        long,
        short,
        global = true,
        action = clap::ArgAction::Count,
        display_order = 1
    )]
    verbose: u8,
    /// Decrease logging verbosity.
    #[arg(
        long,
        short = 'q',
        global = true,
        action = clap::ArgAction::Count,
        conflicts_with = "verbose",
        display_order = 1
    )]
    quiet: u8,
}

impl Verbosity {
    /// Get the filter that should be applied to the logger.
    pub fn filter(&self) -> LevelFilter {
        let mut filter = LevelFilter::Info;
        for _ in 0..self.verbose {
            filter = filter.increment_severity();
        }
        for _ in 0..self.quiet {
            filter = filter.decrement_severity();
        }
        filter
    }
}

const HEADER: Style = AnsiColor::Green.on_default().bold();
const USAGE: Style = AnsiColor::Green.on_default().bold();
const LITERAL: Style = AnsiColor::White.on_default().bold();
const PLACEHOLDER: Style = AnsiColor::Magenta.on_default();
const ERROR: Style = AnsiColor::BrightRed.on_default().bold();
const VALID: Style = AnsiColor::White.on_default().bold();
const INVALID: Style = AnsiColor::Yellow.on_default().bold();

const STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(HEADER)
    .usage(USAGE)
    .literal(LITERAL)
    .placeholder(PLACEHOLDER)
    .error(ERROR)
    .valid(VALID)
    .invalid(INVALID);
