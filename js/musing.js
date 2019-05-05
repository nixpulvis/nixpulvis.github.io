window.onload = function() {
  let path = document.getElementById("score").dataset.musicXml;
  var request = new XMLHttpRequest();
  request.open('GET', path);
  request.responseType = 'blob';
  request.onload = function() {
    console.log(request.response);
    var reader = new FileReader();
    reader.readAsText(request.response);
    reader.onload =  function(e) {
      console.log(e.target.result);
      var display = new opensheetmusicdisplay.OpenSheetMusicDisplay('score');
      display.load(e.target.result).then(function() {
        console.log("e.target.result: " + e.target.result);
        display.render();
      });
    };
  };
  request.send();
}
