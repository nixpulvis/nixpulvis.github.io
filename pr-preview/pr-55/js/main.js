document.addEventListener("DOMContentLoaded", () => {
  // Toggle my hidden, ugly mug.
  document.getElementById("lambda").addEventListener("click", (e) => {
    document.getElementById("me").classList.toggle("hidden");
  });

  // Add a "cheat" code.
  Mousetrap.bind('up up down down left right left right b a enter', function() {
    window.location.href = "/pr-preview/pr-55/MTk3ODkK";
  });

  // Link each heading to itself, shown on hover by `.anchor` in style.css. A
  // page's title (`.title`, from the layouts) links to the page itself.
  // Headings can contain their own links, so rather than wrapping the heading
  // in a link, clicks elsewhere on it are forwarded to the anchor.
  const anchored = [];
  document.querySelectorAll(".content :is(h1, h2, h3, h4, h5, h6):is([id], .title)").forEach((heading) => {
    const anchor = document.createElement("a");
    anchor.className = "anchor";
    if (heading.id) {
      anchor.href = "#" + heading.id;
      anchor.setAttribute("aria-label", "Link to this section");
    } else {
      anchor.href = window.location.pathname;
      anchor.setAttribute("aria-label", "Link to this page");
    }
    anchor.textContent = "#";
    heading.prepend(anchor);
    heading.classList.add("anchored");
    anchored.push(heading);
    heading.addEventListener("click", (e) => {
      // Skip clicks on links, and clicks that end a text selection.
      if (e.target.closest("a") || !window.getSelection().isCollapsed) return;
      anchor.click();
    });
  });

  // Mark the headings whose anchor doesn't fit in the margin, as `.cramped`,
  // so style.css can move it into the heading. Measured with every anchor
  // back in the margin, all at once, and again as the window or an opened
  // `details` changes what's shown.
  const fit = () => {
    anchored.forEach((heading) => heading.classList.remove("cramped"));
    anchored
      .filter((heading) => heading.querySelector(":scope > .anchor").getBoundingClientRect().left < 0)
      .forEach((heading) => heading.classList.add("cramped"));
  };
  fit();
  window.addEventListener("resize", fit);
  document.addEventListener("toggle", fit, true);
});
