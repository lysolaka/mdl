use url::Url;

use mdl::fetch;
use mdl::logger;

use std::error::Error;

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
    let u = "https://www.youtube.com/watch?v=UG03kB-Wv_A";
    let url = Url::parse(&u)?;
    let info = fetch::chapters(&url, None)?;
    let info = toml::to_string(&info)?;
    println!("{}", info);
    Ok(())
}
