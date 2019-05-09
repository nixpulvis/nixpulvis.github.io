window.onload = () => {
  let i = 0;
  for (let score of document.getElementsByClassName("score")) {
    let id = `score${i++}`;
    score.id = id;
    let path = score.dataset.musicXml;
    var request = new XMLHttpRequest();
    request.open('GET', path);
    request.responseType = 'blob';
    request.onload = () => {
      var reader = new FileReader();
      reader.readAsText(request.response);
      reader.onload = (event) => {
        renderMusic(id, event.target.result)
      };
    };
    request.send();
  }
}

function renderMusic(id, xml) {
  console.log(id);
  var display = new opensheetmusicdisplay.OpenSheetMusicDisplay(id);
  display.load(xml).then(() => display.render());
}

