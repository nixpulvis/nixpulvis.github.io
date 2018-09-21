// WIP: Design of dynamic pangram link paragraphs, like the existing one, but
// supporting an arbitrarily growing list of "hallucinations" aka pages.
//
// 1. Generate n-many pangrams (sentences with each letter of the alphabet)
// 2. Glue pangrams into a paragraph, and repeat as needed.
// 3. Add a link to each hallucination on a single character of the first
//      letter of the file name.
//
// 1. generatePangram
//    () -> String
// 2. composeParagraph
//    [String] -> String
// 3. chunkForLinks
//    String -> [String]
// 4. _modifies the DOM, calls other functions_
//    () -> ()
//
// The containing DOM element will be a `<div>` with each paragraph
// (unsurprisingly) inserted as a `<p>`.

// TODO: It would be awesome to be able to get this through Jekyll somehow,
// but I'm not sure it's possible. A self hosted situation may help.
const hallucinations = [
  'problems',
  'hackers-band',
  'fairy-tale',
  'love-hate',
  'right',
  'zuckerburg',
  'judge',
  'maxim',
  'wild',
];

function generatePangram() {
  return "Sphinx of black quartz, judge my vow!";
}

function composeParagraph(...pangrams) {
  return pangrams.join(". ").trim()
}

// TODO: Iterate creating pangrams until we have enough letters to cover all
// duplicate hallucinations file names. It will also make sense to create the
// links at this time.
//
// TODO: We'll need to split paragraphs at some point.
function addLinkParagraphs() {
  let pangrams = [...Array(1)].map(() => generatePangram());
  let paragraph = composeParagraph(...pangrams);

  // lol, this algorithm actually needs thought. ignore this madness for now.
  //
  // I bet there's a decent recursive solution for the splitting. We are
  // splitting so we can insert the links, so technically splitting and
  // removing the character at an index.
  let splitMarkers = [];
  hallucinations.forEach((hallucination) => {
    for (let i = 0; i < paragraph.length; i++) {
      if (paragraph.charAt(i) === hallucination.charAt(0)) {
        splitMarkers.push(i);
      }
    }
  });

  // RIP: I need sleep.

  console.log(paragraph);
  console.log(splitMarkers);
}

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

  // Pangram paragraphs for linking to secret pages.
  addLinkParagraphs();

};

