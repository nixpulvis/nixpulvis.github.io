//! The view's colors, from the theme the page's CSS gives ratzilla.

use nixpulvis::Colors;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Window, window};

use crate::Shared;

/// The view's colors, from the theme the page's CSS gives ratzilla, mixed
/// the way a real terminal's are. The site's dark colors when the page has
/// no theme.
pub(crate) fn page_colors() -> Colors {
    theme().unwrap_or(Colors::DARK)
}

/// Takes the page's theme again whenever the browser switches between light
/// and dark, since the CSS picks the theme by the color scheme.
pub(crate) fn follow_color_scheme(app: &Shared) -> Result<(), JsValue> {
    let Some(query) = window()
        .map(|w| w.match_media("(prefers-color-scheme: dark)"))
        .transpose()?
        .flatten()
    else {
        return Ok(());
    };
    let app = app.clone();
    let on_change = Closure::<dyn FnMut()>::new(move || app.borrow_mut().set_colors(page_colors()));
    query.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())?;
    on_change.forget();
    Ok(())
}

/// The text, background and palette of the page's theme.
fn theme() -> Option<Colors> {
    let window = window()?;
    let document = window.document()?;
    // Custom properties come back as written, so a probe has the browser
    // resolve each one to `rgb(...)`. One that isn't set is transparent, which
    // doesn't read as a color.
    let probe = document.create_element("span").ok()?;
    document.document_element()?.append_child(&probe).ok()?;
    let color = |name: &str| {
        let style = format!("color: var(--ratzilla-{name}, transparent)");
        probe.set_attribute("style", &style).ok()?;
        computed_color(&window, &probe)
    };
    let theme = (|| {
        let mut palette = [[0; 3]; 16];
        for (n, entry) in palette.iter_mut().enumerate() {
            *entry = color(&format!("color-{n}"))?;
        }
        Some(Colors::from_terminal(
            color("foreground")?,
            color("background")?,
            palette,
        ))
    })();
    probe.remove();
    theme
}

/// `element`'s color as the browser computes it, from `rgb(r, g, b)`.
fn computed_color(window: &Window, element: &Element) -> Option<[u8; 3]> {
    let css = window
        .get_computed_style(element)
        .ok()??
        .get_property_value("color")
        .ok()?;
    let mut channels = css
        .strip_prefix("rgb(")?
        .strip_suffix(')')?
        .split(',')
        .map(|c| c.trim().parse().ok());
    let color = [channels.next()??, channels.next()??, channels.next()??];
    channels.next().is_none().then_some(color)
}
