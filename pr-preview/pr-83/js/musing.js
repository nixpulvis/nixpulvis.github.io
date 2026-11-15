window.addEventListener('load', function() {
  let score = document.getElementById("score");
  fetch(score.dataset.musicXml)
    .then(response => response.text())
    .then(renderMusic);
});

function renderMusic(xml) {
  var display = new opensheetmusicdisplay.OpenSheetMusicDisplay('score');
  display.load(xml).then(function() {
    display.render();
  });
}
