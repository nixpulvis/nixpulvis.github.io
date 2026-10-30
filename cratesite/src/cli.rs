//! The command line for a site's binary, so a site's `main` is a few lines.
//!
//! ```no_run
//! # use cratesite::Cli;
//! # fn build(options: cratesite::Options) -> Result<(), String> { Ok(()) }
//! fn main() -> std::process::ExitCode {
//!     Cli::new(env!("CARGO_MANIFEST_DIR"))
//!         .rebuild(&["run", "--quiet", "--bin", "site", "--"])
//!         .run(build)
//! }
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use crate::{Options, Server};

/// A site's command line: `build`, `serve` and `help`.
pub struct Cli {
    source: PathBuf,
    rebuild: Vec<&'static str>,
}

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Build the site once.
    Build,
    /// Build the site, serve it on this port, and build it again on
    /// changes.
    Serve(u16),
    Help,
}

impl Cli {
    pub const USAGE: &str = "\
Usage: <site> <command> [options]

Commands:
  build     Build the site into _site
  serve     Build the site, serve it, and build it again on changes
  help      Show this

Options:
  --unpublished      Include pages marked `published: false`
  --baseurl <path>   Serve the site under a path, like /pr-preview/pr-1
  --destination <d>  Write the site somewhere other than _site
  --port <n>         The port to serve on, 4000 unless given";

    /// The command line for the site whose crate is at `source`, usually
    /// `env!("CARGO_MANIFEST_DIR")`.
    pub fn new(source: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            rebuild: Vec::new(),
        }
    }

    /// The arguments to `cargo` that run this binary, up to its command,
    /// which `serve` uses to build the site again after a change.
    pub fn rebuild(mut self, args: &[&'static str]) -> Self {
        self.rebuild = args.to_vec();
        self
    }

    /// Reads the command and options from `args`, without the program's
    /// name.
    pub fn parse(
        &self,
        args: impl IntoIterator<Item = String>,
    ) -> Result<(Command, Options), String> {
        let mut args = args.into_iter();
        let command = args.next().unwrap_or_else(|| "help".to_owned());
        let mut options = Options {
            destination: self.source.join("_site"),
            source: self.source.clone(),
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
                arg => return Err(format!("unknown option `{arg}`")),
            }
        }
        options.baseurl = options.baseurl.trim_end_matches('/').to_owned();
        let command = match command.as_str() {
            "build" => Command::Build,
            "serve" => Command::Serve(port),
            "help" | "--help" | "-h" => Command::Help,
            command => return Err(format!("unknown command `{command}`")),
        };
        Ok((command, options))
    }

    /// Runs the command line from the process's arguments, building the
    /// site with `build`.
    pub fn run(self, build: impl FnOnce(Options) -> Result<(), String>) -> ExitCode {
        let result = match self.parse(std::env::args().skip(1)) {
            Ok((Command::Build, options)) => build(options),
            Ok((Command::Serve(port), options)) => {
                Server::new(options, port, &self.rebuild).run(build)
            }
            Ok((Command::Help, _)) => {
                println!("{}", Self::USAGE);
                Ok(())
            }
            Err(e) => Err(format!("{e}\n\n{}", Self::USAGE)),
        };
        match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        }
    }
}
