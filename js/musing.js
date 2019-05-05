window.onload = function() {
  let path = document.getElementById("score").dataset.musicXml;
  var request = new XMLHttpRequest();
  request.open('GET', path);
  request.responseType = 'blob';
  request.onload = function() {
    var reader = new FileReader();
    reader.readAsText(request.response);
    reader.onload = renderMusic;
  };
  request.send();
}

function renderMusic(event) {
  var display = new opensheetmusicdisplay.OpenSheetMusicDisplay('score');
  display.load(event.target.result).then(function() {
    display.render();
  });
}

