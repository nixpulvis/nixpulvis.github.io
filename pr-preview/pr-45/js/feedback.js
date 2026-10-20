// Feedback on a selection of the page's text, styled by `.feedback` in
// style.css. Selecting text in `.content` offers two choices:
//
// - Suggest privately: a short note sent to me, along with the quote and a
//   link to it. Notes go to `feedback.endpoint` from _config.yml when it's set
//   (a Formspree-style form URL), otherwise they open an email.
// - Comment publicly: a Hypothesis annotation, shown inline to everyone. This
//   drives the toolbar Hypothesis shows on a selection, which style.css hides.
(() => {
  const ENDPOINT = null;
  const EMAIL = "nathan@nixpulvis.com";

  const menu = document.createElement("div");
  menu.className = "feedback";
  menu.hidden = true;

  // The selection the menu was opened for, kept once the form takes focus.
  let quote = "";
  let anchor = null;
  let writing = false;
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

  // The closest heading at or before the selection, to link near it.
  function headingBefore(node) {
    const headings = document.querySelectorAll(".content :is(h1, h2, h3, h4, h5, h6)[id]");
    let found = null;
    headings.forEach((heading) => {
      if (heading.compareDocumentPosition(node) & Node.DOCUMENT_POSITION_FOLLOWING) {
        found = heading;
      }
    });
    return found;
  }

  // A link that scrolls to and highlights the quote, as a text fragment.
  function quoteLink() {
    const encode = (text) => encodeURIComponent(text).replace(/-/g, "%2D");
    const words = quote.split(/\s+/);
    const text = words.length > 10
      ? encode(words.slice(0, 5).join(" ")) + "," + encode(words.slice(-5).join(" "))
      : encode(words.join(" "));
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
    writing = false;
    menu.hidden = true;
    menu.replaceChildren();
  }

  function showChoices(range) {
    quote = range.toString().trim().replace(/\s+/g, " ");
    anchor = headingBefore(range.startContainer);
    menu.replaceChildren(
      button("Suggest privately", () => showForm(range)),
      button("Comment publicly", comment)
    );
    place(range);
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

  function showForm(range) {
    writing = true;
    const form = document.createElement("form");

    const quoted = document.createElement("blockquote");
    quoted.textContent = quote.length > 200 ? quote.slice(0, 200) + "…" : quote;

    const message = document.createElement("textarea");
    message.name = "message";
    message.rows = 3;
    message.required = true;
    message.placeholder = "What should change?";

    const email = document.createElement("input");
    email.type = "email";
    email.name = "email";
    email.placeholder = "Your email, for a reply (optional)";

    // A field people don't see, and bots fill in.
    const trap = document.createElement("input");
    trap.type = "text";
    trap.name = "_gotcha";
    trap.tabIndex = -1;
    trap.autocomplete = "off";
    trap.className = "trap";

    const send = document.createElement("button");
    send.type = "submit";
    send.textContent = ENDPOINT ? "Send" : "Email it";

    const actions = document.createElement("div");
    actions.className = "actions";
    actions.append(send, button("Cancel", hide));

    form.append(quoted, message, email, trap, actions);
    form.addEventListener("submit", (e) => {
      e.preventDefault();
      submit(message.value, email.value, trap.value);
    });

    menu.replaceChildren(form);
    place(range);
    message.focus({ preventScroll: true });
  }

  async function submit(message, email, trap) {
    const page = document.title;
    const link = quoteLink();
    const section = anchor ? location.origin + location.pathname + "#" + anchor.id : null;

    if (!ENDPOINT) {
      const body = [`> ${quote}`, "", message, "", link].join("\n");
      location.href = "mailto:" + EMAIL
        + "?subject=" + encodeURIComponent("Suggestion for " + page)
        + "&body=" + encodeURIComponent(body);
      hide();
      return;
    }

    const status = (text) => menu.replaceChildren(document.createTextNode(text));
    try {
      const response = await fetch(ENDPOINT, {
        method: "POST",
        headers: { "Content-Type": "application/json", "Accept": "application/json" },
        body: JSON.stringify({ page, quote, message, email, link, section, _gotcha: trap }),
      });
      if (!response.ok) throw new Error(response.statusText);
      status("Thanks, I got it.");
    } catch {
      status("That didn't send. Try again later, or email " + EMAIL + ".");
    }
    // Still `writing`, so a selection change doesn't hide this early.
    setTimeout(hide, 2500);
  }

  // Like Hypothesis, wait for a mouse selection to finish, and for other
  // selections (keyboard, touch handles) to settle.
  function update(delay) {
    clearTimeout(pending);
    pending = setTimeout(() => {
      if (writing) return;
      const range = selectedRange();
      range ? showChoices(range) : hide();
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
