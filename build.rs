//! Makes a module of every page, with cratesite's build script helper.

#[path = "src/config.rs"]
mod config;

fn main() {
    println!("cargo:rerun-if-changed=src/config.rs");
    cratesite::BuildScript::new(&config::CONFIG).run();
}
