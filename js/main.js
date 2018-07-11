window.onload = () => {
  // Toggle my hidden, ugly mug.
  document.getElementById("lambda").addEventListener("click", (e) => {
    document.getElementById("me").classList.toggle("hidden");
  });

  // Scroll just below the header.
  // NOTE: Maybe just on ramblings and projects?
  if (window.pageYOffset < 90) {
    window.scroll(0, 90);
  }

  // Add a "cheat" code.
  Mousetrap.bind('up up down down left right left right b a enter', function() {
    window.location.href = "/MTk3ODkK";
  });
};

