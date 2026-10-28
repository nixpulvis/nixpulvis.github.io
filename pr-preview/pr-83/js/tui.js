// The terminal view: this site's pages drawn with Ratatui, compiled to
// WebAssembly from tui/ in the repository. Switching it on remembers the
// choice and draws over the page. Leaving it, with `q` or the ×, reloads the
// page as normal HTML.

const root = document.documentElement;
const base = root.dataset.baseurl || "";

async function start() {
  root.classList.add("tui");
  let element = document.getElementById("tui");
  if (!element) {
    element = document.createElement("div");
    element.id = "tui";
    document.body.append(element);
  }
  try {
    const tui = await import(`${base}/tui/tui.js`);
    await tui.default();
    tui.start(`${base}/`, location.pathname);
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

document.getElementById("tui-toggle")?.addEventListener("click", (event) => {
  event.preventDefault();
  try {
    localStorage.setItem("tui", "on");
  } catch (e) {}
  start();
});

if (root.classList.contains("tui")) {
  start();
}
