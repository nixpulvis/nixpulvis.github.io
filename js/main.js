// TODO: Make this dynamic
const HEADER_HEIGHT = 90;

function scroll_past_header() {
  // Don't scroll if we're on the home page,
  // or if we're past the point anyway.
  if (window.location.pathname === "/" ||
      window.pageYOffset >= HEADER_HEIGHT)
  {
    return;
  }

  // Scroll with a smooth animation so the user can see there's a nav header
  // above them.
  window.scrollTo({
    top: HEADER_HEIGHT,
    behavior: "smooth"
  })
}

window.onload = () => {
  // Toggle my hidden, ugly mug.
  document.getElementById("lambda").addEventListener("click", (e) => {
    document.getElementById("me").classList.toggle("hidden");
  });

  // Scroll just below the header on each page but the home page. We will
  // leave the home page with the navigation header as it's a common use
  // to navigate from, and it'll help teach the strange behavior.
  scroll_past_header();

  // Add a "cheat" code.
  Mousetrap.bind('up up down down left right left right b a enter', function() {
    window.location.href = "/MTk3ODkK";
  });
};

