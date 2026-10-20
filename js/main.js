window.onload = () => {
  // Toggle my hidden, ugly mug.
  document.getElementById("lambda").addEventListener("click", (e) => {
    document.getElementById("me").classList.toggle("hidden");
  });

  // Add a "cheat" code.
  Mousetrap.bind('up up down down left right left right b a enter', function() {
    window.location.href = "/MTk3ODkK";
  });

  // Link each heading to itself, shown on hover by `.anchor` in style.css. A
  // page's title (`.title`, from the layouts) links to the page itself.
  // Headings can contain their own links, so rather than wrapping the heading
  // in a link, clicks elsewhere on it are forwarded to the anchor.
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
    heading.addEventListener("click", (e) => {
      // Skip clicks on links, and clicks that end a text selection.
      if (e.target.closest("a") || !window.getSelection().isCollapsed) return;
      anchor.click();
    });
  });
};
