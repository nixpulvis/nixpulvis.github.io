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
    tui.start(`${base}/`, location.pathname);
    // So keys work straight away, without a click first.
    element.focus();
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

const toggle = document.getElementById("tui-toggle");
toggle?.addEventListener("click", open);

// Backtick anywhere opens it too, unless it's being typed somewhere.
document.addEventListener("keydown", (event) => {
  if (event.key !== "`" || event.ctrlKey || event.metaKey || event.altKey) return;
  if (event.target.closest("input, textarea, [contenteditable]")) return;
  if (root.classList.contains("tui")) return;
  event.preventDefault();
  open();
});

// The prompt types out its command, `less <page>.md`, on hover or focus, so
// it says what it does before it's clicked. A first visit plays it once on
// its own, for phones with no hover and anyone who'd never think to try.
const command = toggle?.querySelector(".command");
if (command) {
  const text = command.dataset.command;
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  let timer;

  const type = () => {
    clearInterval(timer);
    if (still) {
      command.textContent = text;
      return;
    }
    timer = setInterval(() => {
      command.textContent = text.slice(0, command.textContent.length + 1);
      if (command.textContent === text) clearInterval(timer);
    }, 45);
  };
  const erase = () => {
    clearInterval(timer);
    command.textContent = "";
  };

  toggle.addEventListener("mouseenter", type);
  toggle.addEventListener("focus", type);
  toggle.addEventListener("mouseleave", () => {
    if (document.activeElement !== toggle) erase();
  });
  toggle.addEventListener("blur", erase);

  let seen = true;
  try {
    seen = localStorage.getItem("tui-seen") !== null;
    localStorage.setItem("tui-seen", "1");
  } catch (e) {}
  if (!seen) {
    setTimeout(type, 1000);
    setTimeout(() => {
      if (!toggle.matches(":hover, :focus")) erase();
    }, 4000);
  }
}

if (root.classList.contains("tui")) {
  start();
}
