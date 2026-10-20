// Feedback on a selection of the page's text, styled by `.feedback` in
// style.css. Selecting text in `.content` offers two choices:
//
// - Suggest privately: an email to `feedback.email` from _config.yml, with
//   the quote and a link to it.
// - Comment publicly: a Hypothesis annotation, shown inline to everyone. This
//   drives the toolbar Hypothesis shows on a selection, which style.css hides.
//
// Hypothesis, with its highlights and sidebar, only loads once a reader shows
// comments (`.comments-toggle`, in the rambling layout's byline, with a count
// of them from Hypothesis's API) or comments publicly, and stays on for them
// until they hide comments again. On phones its sidebar, which
// otherwise keeps a toolbar on the screen's edge, opens in `.comments-panel`
// along the bottom instead.
(() => {
  const EMAIL = "nathan@nixpulvis.com";
  const STORAGE_KEY = "comments";
  const PHONE = window.matchMedia("(max-width: 700px)");

  const menu = document.createElement("div");
  menu.className = "feedback";
  menu.hidden = true;

  // Hypothesis's sidebar on phones, shown when it opens. Closing it here only
  // hides it, and Hypothesis shows it again the next time it opens.
  const panel = document.createElement("div");
  panel.className = "comments-panel";
  panel.hidden = true;
  const close = document.createElement("button");
  close.type = "button";
  close.className = "close";
  close.textContent = "Close comments";
  close.addEventListener("click", () => { panel.hidden = true; });
  const frame = document.createElement("div");
  frame.className = "frame";
  panel.append(close, frame);

  let mouseDown = false;
  let pending = null;
  let loading = null;
  let script = null;
  // How many public comments the page has, once Hypothesis's API says.
  let count = null;

  // Remembered per browser; storage can be unavailable, e.g. when blocked.
  function remember(on) {
    try {
      on ? localStorage.setItem(STORAGE_KEY, "on") : localStorage.removeItem(STORAGE_KEY);
    } catch {}
  }

  function remembered() {
    try {
      return localStorage.getItem(STORAGE_KEY) === "on";
    } catch {
      return false;
    }
  }

  // Load Hypothesis, resolving with its Annotate button once it's ready.
  function loadComments() {
    if (loading) return loading;
    window.hypothesisConfig = () => PHONE.matches
      ? {
          openSidebar: false,
          externalContainerSelector: ".comments-panel .frame",
          onLayoutChange: (layout) => { panel.hidden = !layout.expanded; },
        }
      : { openSidebar: false };
    script = document.createElement("script");
    script.src = "https://hypothes.is/embed.js";
    script.async = true;
    loading = new Promise((resolve, reject) => {
      script.addEventListener("error", reject);
      const started = Date.now();
      const poll = setInterval(() => {
        const adder = document.querySelector("hypothesis-adder");
        const annotate = adder?.shadowRoot?.querySelector('button[title^="Annotate"]');
        if (annotate) {
          clearInterval(poll);
          resolve(annotate);
        } else if (Date.now() - started > 15000) {
          clearInterval(poll);
          reject(new Error("Hypothesis didn't load"));
        }
      }, 100);
    });
    loading.catch(() => {});
    document.head.append(script);
    return loading;
  }

  // Unload Hypothesis, as its browser extension does: its highlights, sidebar
  // and the files it added all go, and it can load again later.
  function unloadComments() {
    document.querySelector('link[type="application/annotator+html"][rel="sidebar"]')
      ?.dispatchEvent(new Event("destroy"));
    script?.remove();
    script = null;
    loading = null;
    panel.hidden = true;
  }

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
    const saved = range.cloneRange();
    menu.replaceChildren(
      button("Suggest privately", () => email(quote)),
      button("Comment publicly", () => comment(saved))
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

  async function comment(range) {
    const wasLoaded = !!loading;
    menu.replaceChildren(document.createTextNode("Loading comments…"));
    let annotate;
    try {
      annotate = await loadComments();
    } catch {
      menu.replaceChildren(document.createTextNode(
        "Public comments couldn't load in this browser."
      ));
      return;
    }
    remember(true);
    updateToggle();
    if (!wasLoaded) {
      // Hypothesis only sees selections made after it loads, so make this one
      // again (a tap on the menu may also have dropped it on touch screens),
      // and give it time to notice.
      const selection = document.getSelection();
      selection.removeAllRanges();
      selection.addRange(range);
      await new Promise((resolve) => setTimeout(resolve, 300));
    }
    annotate.click();
    hide();
  }

  function updateToggle() {
    const controls = document.querySelector(".comments-controls");
    if (!controls) return;
    controls.hidden = false;
    const label = (loading ? "Hide comments" : "Show comments") + (count === null ? "" : ` (${count})`);
    controls.querySelector(".comments-toggle").textContent = label;
    controls.querySelector(".comments-open").hidden = !(loading && PHONE.matches);
  }

  // How many public comments the page has, without loading Hypothesis.
  async function countComments() {
    try {
      const uri = encodeURIComponent(location.origin + location.pathname);
      const response = await fetch("https://hypothes.is/api/badge?uri=" + uri);
      count = (await response.json()).total || 0;
      updateToggle();
    } catch {}
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
    document.body.append(menu, panel);

    if (remembered()) loadComments();
    updateToggle();
    countComments();
    document.querySelector(".comments-open")?.addEventListener("click", () => {
      panel.hidden = false;
    });
    document.querySelector(".comments-toggle")?.addEventListener("click", () => {
      if (loading) {
        remember(false);
        unloadComments();
      } else {
        remember(true);
        loadComments();
      }
      updateToggle();
    });
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
