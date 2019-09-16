window.onload = () => {
  // Toggle my hidden, ugly mug.
  document.getElementById("lambda").addEventListener("click", (e) => {
    document.getElementById("me").classList.toggle("hidden");
  });

  // Silly page minimap.
  pagemap(document.querySelector('#map'), {
    viewport: null,
    styles: {
        'header,footer,section,article': 'rgba(0,0,0,0.08)',
        'h1,a': 'rgba(0,0,0,0.10)',
        'h2,h3,h4': 'rgba(0,0,0,0.08)'
    },
    back: 'rgba(0,0,0,0.02)',
    view: 'rgba(0,0,0,0.05)',
    drag: 'rgba(0,0,0,0.10)',
    interval: null
  });

  // Add a "cheat" code.
  Mousetrap.bind('up up down down left right left right b a enter', function() {
    window.location.href = "/MTk3ODkK";
  });
};
