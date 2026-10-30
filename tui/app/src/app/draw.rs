//! Drawing the app: the nav on top, the page in a box, and hints below.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratmark::MarkdownView;
use web_time::Duration;

use super::App;
use crate::Host;
use crate::colors::{BACKGROUND, BORDER, FAINT, MUTED, TEXT, YELLOW};
use crate::link::{Target, same_page};
use crate::page::Page;

/// How long a notice shows before the key hints come back.
const NOTICE_TIME: Duration = Duration::from_secs(3);

/// Widest the page's text gets, for comfortable reading.
const MAX_WIDTH: u16 = 84;

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::new().style(Style::new().bg(BACKGROUND).fg(TEXT)),
            area,
        );
        if self.starting() {
            return;
        }
        let footer = self.footer(area.width);
        let [header, body, bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(u16::from(footer.is_some())),
        ])
        .areas(area);
        if let Some(footer) = footer {
            frame.render_widget(
                Paragraph::new(footer)
                    .style(Style::new().fg(MUTED))
                    .centered(),
                bottom,
            );
        }
        self.draw_header(frame, column(header));
        self.draw_page(frame, column(body));
    }

    /// The line at the bottom. The page loading, messages and notices always
    /// show. The key hints only show when they fit whole, and the page takes
    /// their line when they don't.
    fn footer(&mut self, width: u16) -> Option<String> {
        if self
            .notice
            .as_ref()
            .is_some_and(|(_, at)| at.elapsed() > NOTICE_TIME)
        {
            self.notice = None;
        }
        if let Some(loading) = &self.loading {
            return Some(format!("Loading {}…", loading.path));
        }
        match (&self.message, &self.notice) {
            (Some(message), _) | (None, Some((message, _))) => Some(message.clone()),
            (None, None) => {
                Some(self.hint()).filter(|h| Span::raw(h.as_str()).width() <= usize::from(width))
            }
        }
    }

    /// The key hints, with as many number keys as there are pages in the
    /// nav.
    fn hint(&self) -> String {
        let pages = match self.nav().len().min(9) {
            0 | 1 => String::new(),
            n => format!(" · 1-{n} pages"),
        };
        let quit = match self.host {
            Host::Terminal => "quit",
            Host::Browser => "leave",
        };
        format!("j/k scroll · tab links · enter open · h back{pages} · q {quit}")
    }

    fn draw_header(&mut self, frame: &mut Frame, area: Rect) {
        let current = self.page.as_ref().map_or("", |p| p.path.as_str());
        let mut spans = vec![Span::raw(" ")];
        let mut x = area.x + 1;
        self.drawn.nav.clear();
        for (i, item) in self.nav().into_iter().enumerate() {
            let width = Span::raw(&item.label).width() as u16;
            let style = if i == 0 {
                Style::new().fg(TEXT).bold()
            } else if same_page(current, &item.path) {
                Style::new().fg(YELLOW).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::new().fg(MUTED)
            };
            spans.push(Span::styled(item.label, style));
            spans.push(Span::raw("  "));
            self.drawn
                .nav
                .push((Rect::new(x, area.y, width, 1), item.path));
            x += width + 2;
        }
        frame.render_widget(Line::from(spans), area);

        // A way out for touch screens, with no keyboard for `q`.
        self.drawn.close = Rect::new(area.right().saturating_sub(3), area.y, 3, 1);
        frame.render_widget(
            Line::from(" × ").style(Style::new().fg(MUTED)),
            self.drawn.close,
        );

        #[cfg(feature = "native-links")]
        {
            let buffer = frame.buffer_mut();
            for (rect, path) in &self.drawn.nav {
                buffer.set_hyperlink(
                    *rect,
                    Some(&ratatui::buffer::Hyperlink::new(self.url(path))),
                );
            }
            // A link, so it looks clickable, to this page as plain HTML. A
            // plain click leaves, like `q`.
            let link = ratatui::buffer::Hyperlink::new(self.url(current));
            buffer.set_hyperlink(self.drawn.close, Some(&link));
        }
    }

    fn draw_page(&mut self, frame: &mut Frame, column: Rect) {
        let block = self.page_box(column.width);
        let inner = block.inner(column);
        self.drawn.text = inner;
        frame.render_widget(block, column);

        // Only when the first page failed to load, which the footer says.
        let Some(page) = &mut self.page else {
            return;
        };
        frame.render_stateful_widget(MarkdownView::new(&self.theme), inner, &mut page.view);
        self.fade_draft_links(frame, inner);
        self.draw_scrollbar(frame, column.inner(Margin::new(0, 1)), inner.height);
    }

    /// The box around the page, with its title on top and the line under a
    /// post at the bottom, for a box `width` wide.
    fn page_box(&self, width: u16) -> Block<'static> {
        let block = Block::new()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .border_style(Style::new().fg(BORDER));
        let Some(page) = &self.page else {
            return block;
        };
        // A page the nav names in words is already underlined there.
        let named = self
            .nav()
            .iter()
            .any(|item| item.names_page && same_page(&page.path, &item.path));
        let block = if named {
            block
        } else {
            block.title(page_title(page).centered())
        };
        // Inside the corners, with a space either side.
        match page.meta(usize::from(width).saturating_sub(4)) {
            Some(meta) => block.title_bottom(
                Line::from(format!(" {meta} "))
                    .style(Style::new().fg(MUTED))
                    .centered(),
            ),
            None => block,
        }
    }

    /// Fades the links to drafts, like drafts in the site's lists, on the
    /// page drawn in `inner`.
    fn fade_draft_links(&self, frame: &mut Frame, inner: Rect) {
        let (Some(site), Some(page)) = (&self.site, &self.page) else {
            return;
        };
        let Some(rendered) = page.view.rendered() else {
            return;
        };
        let scroll = page.view.scroll();
        for link in &rendered.links {
            let Target::Page { path, .. } = Target::resolve(&link.url, &page.path, &self.origin)
            else {
                continue;
            };
            if !site.is_draft(path.strip_prefix(&self.base).unwrap_or(&path)) {
                continue;
            }
            for region in &link.regions {
                let Some(row) = region
                    .line
                    .checked_sub(scroll)
                    .filter(|row| *row < usize::from(inner.height))
                else {
                    continue;
                };
                let end = region.end.min(inner.width);
                let start = region.start.min(end);
                let rect = Rect::new(inner.x + start, inner.y + row as u16, end - start, 1);
                frame.buffer_mut().set_style(rect, Style::new().fg(FAINT));
            }
        }
    }

    /// Just a heavier stretch of the box's right border in `area`, with no
    /// track, so the border shows through. Only there when the page is
    /// longer than its `height`.
    fn draw_scrollbar(&self, frame: &mut Frame, area: Rect, height: u16) {
        let Some(page) = &self.page else {
            return;
        };
        let lines = page.view.rendered().map_or(0, |r| r.lines.len());
        let max = lines.saturating_sub(usize::from(height));
        if max == 0 {
            return;
        }
        let mut state = ScrollbarState::new(max).position(page.view.scroll());
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(None)
                .thumb_symbol("┃")
                .thumb_style(Style::new().fg(MUTED)),
            area,
            &mut state,
        );
    }
}

/// A page's title for the top of its box, with a project's status after
/// it. Drafts are dimmed, like they are on the site.
fn page_title(page: &Page) -> Line<'static> {
    let (title, style) = if page.draft {
        (format!(" Draft: {} ", page.title), Style::new().fg(FAINT))
    } else {
        (format!(" {} ", page.title), Style::new().fg(TEXT))
    };
    let status = page
        .status
        .as_ref()
        .map(|s| Span::styled(format!("{s} "), Style::new().fg(MUTED)));
    Line::from_iter(std::iter::once(Span::styled(title, style.bold())).chain(status))
}

/// The middle of `area`, as wide as a page's box, so the nav lines up with it.
fn column(area: Rect) -> Rect {
    let width = area.width.min(MAX_WIDTH + 4);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y,
        width,
        area.height,
    )
}
