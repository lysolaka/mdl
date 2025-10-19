use std::path::PathBuf;

use itertools::EitherOrBoth;
use itertools::Itertools;
use url::Url;

use mdl::cli;
use mdl::cli::FetchEnum;
use mdl::fetch;
use mdl::specfile::Spec;

pub fn fetch_impl(args: &cli::FetchArgs) -> mdl::Result<()> {
    let mapping = args.url.iter().zip_longest(args.id.iter());

    match args.fetch_mode() {
        FetchEnum::Chapters => fetch_chapters(args.outdir.as_ref(), mapping),
        FetchEnum::ChaptersRecursive => fetch_chapters_recursive(args.outdir.as_ref(), mapping),
        FetchEnum::Playlist => fetch_playlist(args.outdir.as_ref(), mapping),
    }
}

pub fn download_impl(args: &cli::DownloadArgs) -> mdl::Result<()> {
    for spec in args.spec.iter() {
        match Spec::read_from(spec)? {
            Spec::Playlist(playlist) => playlist.download(args.outdir.as_ref())?,
            Spec::Chapters(chapters) => chapters.download(args.outdir.as_ref())?,
        };
    }
    Ok(())
}

pub fn tag_impl(args: &cli::TagArgs) -> mdl::Result<()> {
    for spec in args.spec.iter() {
        match Spec::read_from(spec)? {
            Spec::Playlist(playlist) => {
                playlist.postprocess(args.target, args.keep, args.indir.as_ref())?;
                playlist.tag(args.target, args.indir.as_ref(), args.outdir.as_ref())?;
            }
            Spec::Chapters(chapters) => {
                chapters.postprocess(args.target, args.keep, args.indir.as_ref())?;
                chapters.tag(args.target, args.indir.as_ref(), args.outdir.as_ref())?;
            }
        }
    }
    Ok(())
}

fn fetch_chapters<'a, I>(outdir: Option<&PathBuf>, mapping: I) -> mdl::Result<()>
where
    I: Iterator<Item = EitherOrBoth<&'a Url, &'a String>>,
{
    for pair in mapping {
        log::trace!("Fetching mapped pair {:?}", pair);
        match pair {
            EitherOrBoth::Both(url, id) => {
                let spec = fetch::chapters(url, Some(id))?;
                spec.write_to(outdir)?;
            }
            EitherOrBoth::Left(url) => {
                let spec = fetch::chapters(url, None)?;
                spec.write_to(outdir)?;
            }
            EitherOrBoth::Right(id) => log::warn!("ID {} left unused", id),
        }
    }
    Ok(())
}

fn fetch_chapters_recursive<'a, I>(outdir: Option<&PathBuf>, mapping: I) -> mdl::Result<()>
where
    I: Iterator<Item = EitherOrBoth<&'a Url, &'a String>>,
{
    for pair in mapping {
        log::trace!("Fetching mapped pair {:?}", pair);
        match pair {
            EitherOrBoth::Both(url, id) => {
                let specs = fetch::chapters_recursive(url, Some(id))?;
                for spec in specs {
                    spec.write_to(outdir)?;
                }
            }
            EitherOrBoth::Left(url) => {
                let specs = fetch::chapters_recursive(url, None)?;
                for spec in specs {
                    spec.write_to(outdir)?;
                }
            }
            EitherOrBoth::Right(id) => log::warn!("ID {} left unused", id),
        }
    }
    Ok(())
}

fn fetch_playlist<'a, I>(outdir: Option<&PathBuf>, mapping: I) -> mdl::Result<()>
where
    I: Iterator<Item = EitherOrBoth<&'a Url, &'a String>>,
{
    for pair in mapping {
        log::trace!("Fetching mapped pair {:?}", pair);
        match pair {
            EitherOrBoth::Both(url, id) => {
                let spec = fetch::playlist(url, Some(id))?;
                spec.write_to(outdir)?;
            }
            EitherOrBoth::Left(url) => {
                let spec = fetch::playlist(url, None)?;
                spec.write_to(outdir)?;
            }
            EitherOrBoth::Right(id) => log::warn!("ID {} left unused", id),
        }
    }
    Ok(())
}
