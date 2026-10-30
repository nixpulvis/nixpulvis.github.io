//! Read nixpulvis.com in your terminal.
//!
//!     nixpulvis-tui [PATH] [--url URL]
//!
//! PATH is a page on the site, like `/ramblings`. `--url` points at another
//! copy of the site, like a pull request preview.

use std::io;
use std::panic;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use nixpulvis::{App, Effect, Host, Key};
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::crossterm::execute;

const USAGE: &str = "usage: nixpulvis-tui [PATH] [--url URL]";

/// How long a page has to arrive before giving up on it.
const TIMEOUT: Duration = Duration::from_secs(15);

fn main() -> io::Result<()> {
    let args = match Args::parse(std::env::args().skip(1)) {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("{USAGE}");
            return Ok(());
        }
        Err(e) => {
            eprintln!("{e}\n{USAGE}");
            std::process::exit(2);
        }
    };
    let mut app = App::new(Host::Terminal, &args.origin, &args.base);
    let effects = app.start(&args.path);
    let fetcher = Fetcher::new(&args.origin);

    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture)?;
    // Ratatui's own hook puts the terminal back after a panic, but not the
    // mouse.
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = execute!(io::stdout(), DisableMouseCapture);
        hook(info);
    }));
    let result = run(&mut terminal, &mut app, &fetcher, effects);
    execute!(io::stdout(), DisableMouseCapture)?;
    ratatui::restore();
    result
}

/// Draws the app and carries out what it asks, until it quits.
fn run(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    fetcher: &Fetcher,
    mut effects: Vec<Effect>,
) -> io::Result<()> {
    loop {
        for effect in effects.drain(..) {
            match effect {
                Effect::FetchSite(path) => fetcher.site(&path),
                Effect::FetchPage { path, source } => fetcher.page(path, &source),
                Effect::Open(url) => _ = open::that_detached(&url),
                // The app keeps the history itself in a terminal.
                Effect::Navigate(_) | Effect::Back => {}
                Effect::Quit => return Ok(()),
            }
        }
        terminal.draw(|frame| app.draw(frame))?;
        effects.extend(fetcher.deliver(app));
        // Often enough to draw what arrives, and to clear old notices.
        if event::poll(Duration::from_millis(50))? {
            effects.extend(input(app, event::read()?));
        }
    }
}

/// What to read, from the command line.
struct Args {
    /// Like `https://nixpulvis.com`.
    origin: String,
    /// The path the site lives under, `/` except in previews.
    base: String,
    /// The page to start on.
    path: String,
}

impl Args {
    /// The arguments, or `None` when they ask for help.
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Option<Self>, String> {
        let mut url = "https://nixpulvis.com".to_owned();
        let mut page = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--url" => url = args.next().ok_or("--url needs a value")?,
                "-h" | "--help" => return Ok(None),
                flag if flag.starts_with('-') => return Err(format!("unknown option {flag}")),
                _ => page = Some(arg),
            }
        }
        // Split `https://host/base` into the origin and the path the site
        // lives under.
        let after_scheme = url.find("://").map_or(0, |i| i + 3);
        let (origin, base) = match url[after_scheme..].find('/') {
            Some(i) => url.split_at(after_scheme + i),
            None => (url.as_str(), "/"),
        };
        let base = base.trim_end_matches('/');
        Ok(Some(Self {
            origin: origin.to_owned(),
            base: format!("{base}/"),
            path: match page {
                Some(page) => format!("{base}/{}", page.trim_start_matches('/')),
                None => format!("{base}/"),
            },
        }))
    }
}

/// Fetches what the app asks for in the background, and hands it over as it
/// arrives.
struct Fetcher {
    agent: ureq::Agent,
    origin: String,
    send: Sender<Fetched>,
    receive: Receiver<Fetched>,
}

enum Fetched {
    Site(Result<String, String>),
    Page(String, Result<String, String>),
}

impl Fetcher {
    fn new(origin: &str) -> Self {
        let (send, receive) = mpsc::channel();
        Self {
            agent: ureq::Agent::config_builder()
                .timeout_global(Some(TIMEOUT))
                .build()
                .into(),
            origin: origin.to_owned(),
            send,
            receive,
        }
    }

    fn site(&self, path: &str) {
        self.get(path, Fetched::Site);
    }

    fn page(&self, path: String, source: &str) {
        self.get(source, move |markdown| Fetched::Page(path, markdown));
    }

    /// Passes everything that's arrived to `app`, with what it asks for
    /// next.
    fn deliver(&self, app: &mut App) -> Vec<Effect> {
        let mut effects = Vec::new();
        while let Ok(fetched) = self.receive.try_recv() {
            match fetched {
                Fetched::Site(json) => app.site_loaded(json),
                Fetched::Page(path, markdown) => effects.extend(app.page_loaded(&path, markdown)),
            }
        }
        effects
    }

    fn get(
        &self,
        path: &str,
        fetched: impl FnOnce(Result<String, String>) -> Fetched + Send + 'static,
    ) {
        let (agent, send) = (self.agent.clone(), self.send.clone());
        let url = format!("{}{path}", self.origin);
        thread::spawn(move || {
            let body = agent
                .get(&url)
                .call()
                .and_then(|mut response| response.body_mut().read_to_string())
                .map_err(|e| e.to_string());
            send.send(fetched(body))
        });
    }
}

/// What a terminal event does to the app.
fn input(app: &mut App, event: Event) -> Vec<Effect> {
    match event {
        Event::Key(key) if key.kind != KeyEventKind::Release => match translate(key) {
            Some(key) => app.key(key),
            None if is_ctrl_c(key) => vec![Effect::Quit],
            None => Vec::new(),
        },
        Event::Mouse(MouseEvent {
            kind, column, row, ..
        }) => {
            match kind {
                MouseEventKind::Down(MouseButton::Left) => return app.click(column, row),
                MouseEventKind::ScrollDown => _ = app.scroll_by(3),
                MouseEventKind::ScrollUp => _ = app.scroll_by(-3),
                MouseEventKind::ScrollRight => _ = app.shift_at(column, row, 3),
                MouseEventKind::ScrollLeft => _ = app.shift_at(column, row, -3),
                _ => {}
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

fn translate(key: KeyEvent) -> Option<Key> {
    Some(match key.code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace | KeyCode::Left => Key::Back,
        KeyCode::Esc => Key::Esc,
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            Key::Char(c)
        }
        _ => return None,
    })
}

fn is_ctrl_c(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Option<Args>, String> {
        Args::parse(args.iter().map(|a| a.to_string()))
    }

    #[test]
    fn args() {
        let args = parse(&[]).unwrap().unwrap();
        assert_eq!(
            (args.origin.as_str(), args.base.as_str(), args.path.as_str()),
            ("https://nixpulvis.com", "/", "/")
        );
        let args = parse(&["ramblings", "--url", "http://localhost:4000/pr/1/"])
            .unwrap()
            .unwrap();
        assert_eq!(
            (args.origin.as_str(), args.base.as_str(), args.path.as_str()),
            ("http://localhost:4000", "/pr/1/", "/pr/1/ramblings")
        );
        assert!(parse(&["--help"]).unwrap().is_none());
        assert!(parse(&["--nope"]).is_err());
        assert!(parse(&["--url"]).is_err());
    }
}
