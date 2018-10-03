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
  'fairy-tale',
  'hackers-band',
  'love-hate',
  'maxim',
  'passion',
  'problems',
  'right',
  'war',
  'who',
  'wild',
  'zuckerburg',
];

function generatePangram() {
  return "Sphinx of black quartz, judge my vow!".repeat(3);
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
  let splitMarkers = {};
  for (let i = 0; i < paragraph.length; i++) {
    let paragraphChar = paragraph.charAt(i);
    hallucinations.forEach((hallucination, index) => {
      if (i in splitMarkers) { return }
      let hallucinationChar = hallucination.charAt(0);
      if (paragraphChar === hallucinationChar) {
        splitMarkers[i] = paragraphChar;
        hallucinations.splice(index, 1);
      }
    });
  }

  // RIP: I need sleep.

  console.log(paragraph);
  console.log(splitMarkers);
}

window.onload = () => {
  addLinkParagraphs()
}
