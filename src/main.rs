use std::error::Error;

use clap::Parser;
use itertools::Itertools;

use mdl::cli::{self, Cli, Subcommand};
use mdl::logger;

mod impls;

const ERR_COLOR: anstyle::Style = anstyle::AnsiColor::BrightRed.on_default().bold();

fn main() {
    let cli = Cli::parse();
    logger::init(cli.verbosity.filter());
    if let Err(e) = main_impl(cli) {
        eprintln!("\n{ERR_COLOR}error{ERR_COLOR:#}: {}", e);

        let mut source = e.source();
        if source.is_some() {
            eprintln!("\nCaused by:");
            let mut i = 0;

            while let Some(err) = source {
                eprintln!("{}: {}", i, err);
                source = err.source();
                i += 1;
            }
        }
        std::process::exit(-1);
    }
}

fn main_impl(args: Cli) -> mdl::Result<()> {
    match args.action {
        Subcommand::Fetch(fetch_args) => fetch_impl(&fetch_args),
        Subcommand::Download(download_args) => todo!(),
        Subcommand::Tag(tag_args) => todo!(),
    }
}

fn fetch_impl(args: &cli::FetchArgs) -> mdl::Result<()> {
    let mapping = args.url.iter().zip_longest(args.id.iter());

    match args.fetch_mode() {
        cli::FetchEnum::Chapters => impls::fetch_chapters(args.outdir.as_ref(), mapping),
        cli::FetchEnum::ChaptersRecursive => impls::fetch_chapters_recursive(args.outdir.as_ref(), mapping),
        cli::FetchEnum::Playlist => impls::fetch_playlist(args.outdir.as_ref(), mapping),
    }
}
