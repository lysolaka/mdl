use std::error::Error;

use clap::Parser;

use mdl::logger;
use mdl::cli::Cli;

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
    println!("{:#?}", args);
    Ok(())
}
