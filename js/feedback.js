---
---
// Feedback on a selection of the page's text, styled by `.feedback` in
// style.css. Selecting text in `.content` offers two choices:
//
// - Suggest privately: an email to `feedback.email` from _config.yml, with
//   the quote and a link to it.
// - Comment publicly: a Hypothesis annotation, shown inline to everyone. This
//   drives the toolbar Hypothesis shows on a selection, which style.css hides.
(() => {
  const EMAIL = {{ site.feedback.email | jsonify }};

  const menu = document.createElement("div");
  menu.className = "feedback";
  menu.hidden = true;

  let mouseDown = false;
  let pending = null;

  function button(label, onClick) {
    const b = document.createElement("button");
    b.type = "button";
    b.textContent = label;
    // Keep the page's selection, which Hypothesis annotates.
    b.addEventListener("mousedown", (e) => e.preventDefault());
    b.addEventListener("click", onClick);
    return b;
  }

  // The selected range, when it's within the page's content.
  function selectedRange() {
    const selection = document.getSelection();
    if (!selection || selection.isCollapsed || !selection.rangeCount) return null;
    const range = selection.getRangeAt(0);
    const content = document.querySelector(".content");
    if (!content || !content.contains(range.commonAncestorContainer)) return null;
    if (!selection.toString().trim()) return null;
    return range;
  }

  // A link that scrolls to and highlights the quote, as a text fragment.
  function quoteLink(quote) {
    const encode = (text) => encodeURIComponent(text).replace(/-/g, "%2D");
    const words = quote.split(" ");
    const text = words.length > 10
      ? encode(words.slice(0, 5).join(" ")) + "," + encode(words.slice(-5).join(" "))
      : encode(quote);
    return location.origin + location.pathname + "#:~:text=" + text;
  }

  function place(range) {
    const rects = range.getClientRects();
    const rect = rects[rects.length - 1] || range.getBoundingClientRect();
    menu.hidden = false;
    const left = Math.min(
      Math.max(rect.right - menu.offsetWidth / 2, 10),
      document.documentElement.clientWidth - menu.offsetWidth - 10
    );
    menu.style.left = left + window.scrollX + "px";
    menu.style.top = rect.bottom + window.scrollY + 8 + "px";
  }

  function hide() {
    menu.hidden = true;
    menu.replaceChildren();
  }

  function show(range) {
    const quote = range.toString().trim().replace(/\s+/g, " ");
    menu.replaceChildren(
      button("Suggest privately", () => email(quote)),
      button("Comment publicly", comment)
    );
    place(range);
  }

  function email(quote) {
    const body = [`> ${quote}`, "", "", quoteLink(quote)].join("\n");
    location.href = "mailto:" + EMAIL
      + "?subject=" + encodeURIComponent("Suggestion for " + document.title)
      + "&body=" + encodeURIComponent(body);
    // Done with the selection, as Hypothesis is after annotating it.
    document.getSelection().removeAllRanges();
    hide();
  }

  function comment() {
    const adder = document.querySelector("hypothesis-adder");
    const annotate = adder?.shadowRoot?.querySelector('button[title^="Annotate"]');
    if (annotate) {
      annotate.click();
      hide();
    } else {
      menu.replaceChildren(document.createTextNode(
        "Public comments are still loading, or blocked by this browser."
      ));
    }
  }

  // Like Hypothesis, wait for a mouse selection to finish, and for other
  // selections (keyboard, touch handles) to settle.
  function update(delay) {
    clearTimeout(pending);
    pending = setTimeout(() => {
      const range = selectedRange();
      range ? show(range) : hide();
    }, delay);
  }

  window.addEventListener("DOMContentLoaded", () => {
    document.body.append(menu);
    document.addEventListener("mousedown", (e) => {
      if (!menu.contains(e.target)) mouseDown = true;
    });
    document.addEventListener("mouseup", () => {
      if (mouseDown) update(10);
      mouseDown = false;
    });
    document.addEventListener("selectionchange", () => {
      if (!mouseDown) update(150);
    });
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !menu.hidden) hide();
    });
  });
})();
