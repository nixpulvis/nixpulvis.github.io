// The terminal view: this site's pages drawn with Ratatui, compiled to
// WebAssembly from tui/ in the repository. Switching it on remembers the
// choice and draws over the page. Leaving it, with `q` or the ×, reloads the
// page as normal HTML.

const root = document.documentElement;
const base = root.dataset.baseurl || "";
// Changes with every build, so browsers don't keep running an old bundle.
const version = root.dataset.build || "";

async function start() {
  root.classList.add("tui");
  let element = document.getElementById("tui");
  if (!element) {
    element = document.createElement("div");
    element.id = "tui";
    document.body.append(element);
  }
  // The view draws in a shadow root, out of sight of the page's own
  // scripts. Ad blockers watch the page for changes and re-scan it, and the
  // view changes on every frame of a scroll.
  const shadow = element.shadowRoot ?? element.attachShadow({ mode: "open" });
  const screen = document.createElement("div");
  screen.className = "screen";
  const style = document.createElement("style");
  // The screen holds focus for its keys, and a ring around the whole
  // window says nothing.
  style.textContent =
    ".screen { height: 100%; outline: none; } pre { margin: 0; }";
  shadow.replaceChildren(style, screen);
  try {
    const tui = await import(`${base}/tui/tui.js?v=${version}`);
    await tui.default({
      module_or_path: `${base}/tui/tui_bg.wasm?v=${version}`,
    });
    // Tab moves between links in the view, so the browser shouldn't also
    // move its focus through them.
    element.addEventListener("keydown", (event) => {
      if (event.key === "Tab") event.preventDefault();
    });
    tui.start(`${base}/`, location.pathname + location.hash, screen);
    // So keys work straight away, without a click first.
    screen.focus();
  } catch (error) {
    // Most likely a build without Rust, which skips the terminal view.
    console.warn("The terminal view isn't available:", error);
    stop();
  }
}

function stop() {
  try {
    localStorage.removeItem("tui");
  } catch (e) {}
  root.classList.remove("tui");
  document.getElementById("tui")?.remove();
}

function open() {
  try {
    localStorage.setItem("tui", "on");
  } catch (e) {}
  start();
}

// The key after the name, `>_`, which opens the view. It's only added here,
// so without JavaScript it isn't shown at all.
const title = document.querySelector("header .title");
if (title) {
  const toggle = document.createElement("button");
  toggle.id = "tui-toggle";
  toggle.type = "button";
  toggle.textContent = ">_";
  toggle.ariaLabel = "Read this page in a terminal";
  toggle.addEventListener("click", open);
  title.append(toggle);
}

// Backtick anywhere opens it too, unless it's being typed somewhere.
document.addEventListener("keydown", (event) => {
  if (event.key !== "`" || event.ctrlKey || event.metaKey || event.altKey) return;
  if (event.target.closest("input, textarea, [contenteditable]")) return;
  if (root.classList.contains("tui")) return;
  event.preventDefault();
  open();
});

if (root.classList.contains("tui")) {
  start();
}
