//! Serving the site while writing it.
//!
//! The pages are compiled into the site's crate, so a change to one means
//! building the crate again. [`Server::run`] serves the site, watches the
//! crate, and on a change runs the site's build command with cargo, which
//! does both.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use crate::Options;

/// Serves the site on localhost, building it again whenever a file changes.
pub struct Server {
    options: Options,
    port: u16,
    /// The arguments to `cargo` that run the site's binary, up to its
    /// `build` command, like `run --bin site --`.
    rebuild: Vec<String>,
}

impl Server {
    /// How often to look for changes.
    const POLL: Duration = Duration::from_millis(500);

    pub fn new(options: Options, port: u16, rebuild: &[&str]) -> Self {
        Self {
            options,
            port,
            rebuild: rebuild.iter().map(|&a| a.to_owned()).collect(),
        }
    }

    /// Builds the site with `build` and serves it until stopped.
    pub fn run(self, build: impl FnOnce(Options) -> Result<(), String>) -> Result<(), String> {
        build(self.options.clone())?;
        let listener = TcpListener::bind(("127.0.0.1", self.port)).map_err(|e| e.to_string())?;
        eprintln!(
            "Serving at http://127.0.0.1:{}{}/",
            self.port, self.options.baseurl
        );
        let root = self.options.destination.clone();
        let baseurl = self.options.baseurl.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = Self::respond(stream, &root, &baseurl);
            }
        });
        let mut seen = self.snapshot();
        loop {
            std::thread::sleep(Self::POLL);
            let now = self.snapshot();
            if now != seen {
                seen = now;
                eprintln!("Building...");
                self.rebuild();
            }
        }
    }

    /// Builds the generator and the site again, in another process, so
    /// changed pages are compiled in.
    fn rebuild(&self) {
        let mut command = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
        command
            .current_dir(&self.options.source)
            .args(&self.rebuild)
            .args(["build", "--destination"])
            .arg(&self.options.destination);
        if !self.options.baseurl.is_empty() {
            command.args(["--baseurl", &self.options.baseurl]);
        }
        if self.options.unpublished {
            command.arg("--unpublished");
        }
        match command.status() {
            Ok(status) if status.success() => eprintln!("Built."),
            Ok(_) => eprintln!("The build failed."),
            Err(e) => eprintln!("Couldn't run cargo: {e}"),
        }
    }

    /// When every file in the crate, outside of builds, last changed.
    fn snapshot(&self) -> BTreeMap<PathBuf, SystemTime> {
        let mut files = BTreeMap::new();
        let skip = [
            self.options.destination.clone(),
            self.options.source.join("target"),
            self.options.source.join(".git"),
            self.options.source.join(".site-cache"),
        ];
        Self::scan(&self.options.source, &skip, &mut files);
        files
    }

    fn scan(dir: &Path, skip: &[PathBuf], files: &mut BTreeMap<PathBuf, SystemTime>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if skip.contains(&path) {
                continue;
            }
            match entry.metadata() {
                Ok(meta) if meta.is_dir() => Self::scan(&path, skip, files),
                Ok(meta) => {
                    files.insert(path, meta.modified().unwrap_or(SystemTime::UNIX_EPOCH));
                }
                Err(_) => {}
            }
        }
    }

    /// Answers one request with a file from `root`, the way GitHub Pages
    /// does: `/foo` is `foo.html`, and `/foo/` is `foo/index.html`.
    fn respond(mut stream: TcpStream, root: &Path, baseurl: &str) -> std::io::Result<()> {
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line)?;
        let target = line.split_whitespace().nth(1).unwrap_or("/");
        let path = target.split(['?', '#']).next().unwrap_or("/");
        let path = Self::decode(path.strip_prefix(baseurl).unwrap_or(path));
        let relative = path.trim_start_matches('/');
        let candidates = if relative.is_empty() || relative.ends_with('/') {
            vec![root.join(relative).join("index.html")]
        } else {
            vec![
                root.join(relative),
                root.join(format!("{relative}.html")),
                root.join(relative).join("index.html"),
            ]
        };
        let found = candidates
            .into_iter()
            .find(|p| p.is_file() && p.starts_with(root));
        let (status, file) = match found {
            Some(file) => ("200 OK", file),
            None => ("404 Not Found", root.join("404.html")),
        };
        let body = fs::read(&file).unwrap_or_default();
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
            Self::content_type(&file),
            body.len()
        )?;
        stream.write_all(&body)
    }

    /// Percent escapes in a URL's path decoded.
    fn decode(path: &str) -> String {
        let bytes = path.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%'
                && let Some(b) = path
                    .get(i + 1..i + 3)
                    .and_then(|h| u8::from_str_radix(h, 16).ok())
            {
                out.push(b);
                i += 3;
                continue;
            }
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    fn content_type(file: &Path) -> &'static str {
        match file.extension().and_then(|e| e.to_str()).unwrap_or("") {
            "html" => "text/html; charset=utf-8",
            "css" => "text/css",
            "js" => "text/javascript",
            "json" => "application/json",
            "md" => "text/markdown; charset=utf-8",
            "txt" => "text/plain; charset=utf-8",
            "xml" => "application/xml",
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            "pdf" => "application/pdf",
            "wasm" => "application/wasm",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            _ => "application/octet-stream",
        }
    }
}
