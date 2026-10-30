//! Builds nixpulvis.com. See `cargo site help`.

use std::process::ExitCode;

use cratesite::{Cli, Theme as _};
use nixpulvis::site::Site;

fn main() -> ExitCode {
    Cli::new(env!("CARGO_MANIFEST_DIR"))
        .rebuild(&[
            "run",
            "--quiet",
            "--package",
            "nixpulvis",
            "--features",
            "site",
            "--bin",
            "site",
            "--",
        ])
        .run(|options| Site::load(options)?.build())
}
