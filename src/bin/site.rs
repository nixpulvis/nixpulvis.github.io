//! Builds nixpulvis.com. See `cargo site help`.

use std::path::PathBuf;
use std::process::ExitCode;

use nixpulvis::site::{Options, Server, Site};

const USAGE: &str = "\
Usage: cargo site <command> [options]

Commands:
  build     Build the site into _site
  serve     Build the site, serve it, and build it again on changes
  help      Show this

Options:
  --unpublished      Include pages marked `published: false`
  --baseurl <path>   Serve the site under a path, like /pr-preview/pr-1
  --destination <d>  Write the site somewhere other than _site
  --port <n>         The port to serve on, 4000 unless given";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_owned());
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut options = Options {
        destination: source.join("_site"),
        source,
        baseurl: String::new(),
        unpublished: false,
    };
    let mut port = 4000;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--unpublished" => options.unpublished = true,
            "--baseurl" => options.baseurl = args.next().unwrap_or_default(),
            "--destination" => options.destination = args.next().unwrap_or_default().into(),
            "--port" => port = args.next().and_then(|p| p.parse().ok()).unwrap_or(port),
            arg => {
                eprintln!("unknown option `{arg}`\n\n{USAGE}");
                return ExitCode::FAILURE;
            }
        }
    }
    options.baseurl = options.baseurl.trim_end_matches('/').to_owned();

    let result = match command.as_str() {
        "build" => Site::load(options).and_then(|mut site| site.build()),
        "serve" => Server::new(options, port).run(),
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        command => Err(format!("unknown command `{command}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
