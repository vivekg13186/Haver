var graph = new LGraph();

var canvas = new LGraphCanvas("#mycanvas", graph);


graph.start()

var canvasElement = document.getElementById("mycanvas");

updateEditorHiPPICanvas();
window.addEventListener("resize", function () {

    updateEditorHiPPICanvas();
});
function updateEditorHiPPICanvas() {
    const ratio = window.devicePixelRatio;

    const rect = canvasElement.parentNode.getBoundingClientRect();
    console.log(rect)
    const { width, height } = rect;
    canvasElement.width = width * ratio;
    canvasElement.height = height * ratio;
    canvasElement.style.width = width + "px";
    canvasElement.style.height = height + "px";
    canvasElement.getContext("2d").scale(ratio, ratio);
    return canvasElement;
}