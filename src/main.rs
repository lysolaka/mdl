use std::error::Error;

use mdl::logger;
use mdl::specfile::Spec;

fn main() {
    logger::init(log::LevelFilter::Debug);
    if let Err(e) = main_impl() {
        eprintln!("\nError: {}", e);

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
    }
}

fn main_impl() -> mdl::Result<()> {
    // let u = "https://soundcloud.com/alcomindz/sets/koldi-kolins-dawaj-mixtape-2018";
    // let url = Url::parse(&u)?;
    // let info = fetch::playlist(&url, Some("dawaj"))?;
    // info.write_to(Some("specs/"))?;
    //
    // let u = "https://www.youtube.com/playlist?list=PLx6F6orIEZdmbMQyQKYXq2R0XLgYk1Lpw";
    // let url = Url::parse(&u)?;
    // let info = fetch::playlist(&url, Some("012"))?;
    // info.write_to(Some("specs/"))?;
    //
    // let u = "https://www.youtube.com/watch?v=vnd35SLG4Yc";
    // let url = Url::parse(&u)?;
    // let info = fetch::chapters(&url, Some("v5-2"))?;
    // info.write_to(Some("specs/"))?;
    //
    // let u = "https://www.youtube.com/playlist?list=PLqWr7dyJNgLK45KSPhhti4FcWLhEWlegt";
    // let url = Url::parse(&u)?;
    // let info = fetch::chapters_recursive(&url, Some("inazuma"))?;
    // for i in info {
    //     i.write_to(Some("specs/"))?;
    // }

    let spec = Spec::read_from("specs/012.toml")?;
    assert!(matches!(spec, Spec::Playlist(_)));

    let spec = Spec::read_from("specs/dawaj.toml")?;
    assert!(matches!(spec, Spec::Playlist(_)));

    let spec = Spec::read_from("specs/v5-2.toml")?;
    assert!(matches!(spec, Spec::Chapters(_)));

    let spec = Spec::read_from("Cargo.toml");
    assert!(spec.is_err());
    Ok(())
}
