//! The terminal view of nixpulvis.com, in the browser.
//!
//! `js/tui.js` on the site loads this and calls [`start`] when the terminal
//! view is switched on. It draws over the page, in `#tui`, and switching it
//! off reloads the page as normal HTML.

use std::cell::RefCell;
use std::rc::Rc;

use nixpulvis::{App, Effect, Host, Key};
use ratzilla::event::{KeyCode, KeyEvent, MouseButton, MouseEventKind};
use ratzilla::ratatui::Terminal;
use ratzilla::{CellSized, DomBackend, WebRenderer};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Response, window};

mod gesture;

use gesture::Gestures;

type Shared = Rc<RefCell<App>>;

/// Where the "on" switch is remembered, shared with `js/tui.js`.
const STORAGE_KEY: &str = "tui";

/// Starts the terminal view in `screen`, on the page at `path`, for a site
/// living under `base`.
#[wasm_bindgen]
pub fn start(base: &str, path: &str, screen: web_sys::Element) -> Result<(), JsValue> {
    let origin = window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    // The switch is remembered, so each page the browser loads opens in the
    // view again.
    let mut app = App::new(Host::Browser, origin, base);
    let effects = app.start(path);
    let app: Shared = Rc::new(RefCell::new(app));
    carry_out(&app, effects);

    let backend = DomBackend::new_by_element(screen.clone()).map_err(js_error)?;
    let mut terminal = Terminal::new(backend).map_err(js_error)?;

    let keys = app.clone();
    terminal
        .on_key_event(move |event| {
            if let Some(key) = translate(&event) {
                let effects = keys.borrow_mut().key(key);
                carry_out(&keys, effects);
            }
        })
        .map_err(js_error)?;

    let mouse = app.clone();
    terminal
        .on_mouse_event({
            // A drag, like one selecting text, ends in a click too, which
            // only counts when it ends where it started.
            let mut down = None;
            move |event| match event.kind {
                MouseEventKind::ButtonDown(MouseButton::Left) => {
                    down = Some((event.col, event.row));
                }
                MouseEventKind::SingleClick(MouseButton::Left)
                    if down == Some((event.col, event.row)) =>
                {
                    let effects = mouse.borrow_mut().click(event.col, event.row);
                    carry_out(&mouse, effects);
                }
                _ => {}
            }
        })
        .map_err(js_error)?;

    // Measured by the backend from the view's font, which is fixed in CSS,
    // so it's measured once.
    let (width, height) = terminal.backend().cell_size_css_px();
    Gestures::listen(&app, &screen, (f64::from(width), f64::from(height)))?;
    follow_links_in_app(&screen)?;

    // Only the page's text can be selected, not the nav, border or hints.
    let selectable = terminal.backend().selectable_area();
    let draw = app.clone();
    terminal.draw_web(move |frame| {
        let mut app = draw.borrow_mut();
        app.draw(frame);
        selectable.set(Some(app.text_area()));
    });
    Ok(())
}

/// Links are real `<a>`s, but a plain click is the app's to follow, so it
/// can scroll to a section or open a link in a new tab. Middle clicks,
/// modified clicks and the context menu are left to the browser.
fn follow_links_in_app(screen: &web_sys::Element) -> Result<(), JsValue> {
    let on_click = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|e: web_sys::MouseEvent| {
        let plain =
            e.button() == 0 && !(e.ctrl_key() || e.meta_key() || e.shift_key() || e.alt_key());
        let link = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|t| t.closest("a").ok().flatten());
        if plain && link.is_some() {
            e.prevent_default();
        }
    });
    screen.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();
    Ok(())
}

fn translate(event: &KeyEvent) -> Option<Key> {
    Some(match event.code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Tab if event.shift => Key::BackTab,
        KeyCode::Tab => Key::Tab,
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace | KeyCode::Left => Key::Back,
        KeyCode::Esc => Key::Esc,
        KeyCode::Char(c) if !event.ctrl && !event.alt => Key::Char(c),
        _ => return None,
    })
}

/// Carries out what the app asks for.
fn carry_out(app: &Shared, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::FetchSite(path) => {
                let app = app.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let json = fetch(&path).await;
                    app.borrow_mut().site_loaded(json);
                });
            }
            Effect::FetchPage { path, source } => {
                let app = app.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let markdown = fetch(&source).await;
                    let effects = app.borrow_mut().page_loaded(&path, markdown);
                    carry_out(&app, effects);
                });
            }
            Effect::Open(url) => {
                if let Some(window) = window() {
                    let _ = window.open_with_url_and_target(&url, "_blank");
                }
            }
            Effect::Navigate(path) => {
                if let Some(window) = window() {
                    let _ = window.location().assign(&path);
                }
            }
            Effect::Back => {
                if let Some(window) = window()
                    && let Ok(history) = window.history()
                {
                    let _ = history.back();
                }
            }
            Effect::Quit => {
                if let Some(window) = window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        let _ = storage.remove_item(STORAGE_KEY);
                    }
                    let _ = window.location().reload();
                }
            }
        }
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

async fn fetch(path: &str) -> Result<String, String> {
    let window = window().ok_or("no window")?;
    // The page's build number, so the browser doesn't hold on to an old
    // copy of the site's Markdown, as it does for the bundle.
    let build = window
        .document()
        .and_then(|d| d.document_element())
        .and_then(|e| e.get_attribute("data-build"));
    let url = match build {
        Some(build) => format!("{path}?v={build}"),
        None => path.to_owned(),
    };
    let response = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let response: Response = response.dyn_into().map_err(|e| format!("{e:?}"))?;
    if !response.ok() {
        return Err(format!("{} {}", response.status(), response.status_text()));
    }
    let text = response.text().map_err(|e| format!("{e:?}"))?;
    let text = JsFuture::from(text).await.map_err(|e| format!("{e:?}"))?;
    text.as_string().ok_or_else(|| "not text".to_owned())
}
