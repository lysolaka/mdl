use std::path::PathBuf;

use itertools::EitherOrBoth;
use url::Url;

use mdl::fetch;

pub fn fetch_chapters<'a, I>(outdir: Option<&PathBuf>, mapping: I) -> mdl::Result<()>
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

pub fn fetch_chapters_recursive<'a, I>(outdir: Option<&PathBuf>, mapping: I) -> mdl::Result<()>
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

pub fn fetch_playlist<'a, I>(outdir: Option<&PathBuf>, mapping: I) -> mdl::Result<()>
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
