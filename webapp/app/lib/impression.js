import { Simplification, Segmentation } from 'visionmagic';
import * as segmentation from "./lib/segmentation";
import * as simplification from "./lib/simplification";

let runner;
const canvas = document.getElementById('frame');
const ctx = canvas.getContext('2d');
const svg = document.getElementById('svg');
const img = new Image();
const watermark = { light: new Image(), dark: new Image() };
const progress = document.getElementById('progressbar');
const progressregion = document.getElementById('progressregion');

watermark.light.src = 'assets/visioncortex-logo-watermark-light.png';
watermark.dark.src = 'assets/visioncortex-logo-watermark-dark.png';

// Hide canas and svg on load
canvas.style.display = 'none';
svg.style.display = 'none';

if (document.getElementById('export')) {
    document.getElementById('export').addEventListener('click', function (e) {
        let filename = 'visionmagic-' + new Date().toISOString().slice(0, 19).replace(/:/g, '').replace('T', ' ') + '.png';

        /// create an "off-screen" anchor tag
        let lnk = document.createElement('a');

        /// the key here is to set the download attribute of the a tag
        lnk.download = filename;

        /// convert canvas content to data-uri for link. When download
        /// attribute is set the content pointed to by link will be
        /// pushed as "download" in HTML5 capable browsers
        lnk.href = canvas.toDataURL("image/png;base64");

        /// create a "fake" click-event to trigger the download
        if (document.createEvent) {
            e = document.createEvent("MouseEvents");
            e.initMouseEvent("click", true, true, window,
                0, 0, 0, 0, 0, false, false, false,
                false, 0, null);

            lnk.dispatchEvent(e);
        } else if (lnk.fireEvent) {
            lnk.fireEvent("onclick");
        }
    }, false);
}

let global =
    document.body.id == 'simplification' ? simplification.global() :
    document.body.id == 'segmentation' ? segmentation.global() :
    null;

let presetConfigs = 
    document.body.id == 'simplification' ? simplification.presetConfigs() :
    document.body.id == 'segmentation' ? segmentation.presetConfigs() :
    null;

document.body.id == 'simplification' ? simplification.setup(global, reconfig, restart) :
document.body.id == 'segmentation' ? segmentation.setup(global, reconfig, restart) :
null;

// Upload button
let imageSelect = document.getElementById('imageSelect'),
imageInput = document.getElementById('imageInput');  
imageSelect.addEventListener('click', function (e) {
    imageInput.click();
    e.preventDefault();
});

imageInput.addEventListener('change', function (e) {
    setSourceAndRestart(this.files[0]);
});

// Drag-n-Drop
let drop = document.getElementById('drop');
let droptext = document.getElementById('droptext');
drop.addEventListener('dragenter', function (e) {
    if (e.preventDefault) e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    droptext.classList.add('hovering');
    return false;
});

drop.addEventListener('dragleave', function (e) {
    if (e.preventDefault) e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    droptext.classList.remove('hovering');
    return false;
});

drop.addEventListener('dragover', function (e) {
    if (e.preventDefault) e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    droptext.classList.add('hovering');
    return false;
});

drop.addEventListener('drop', function (e) {
    if (e.preventDefault) e.preventDefault();
    droptext.classList.remove('hovering');
    setSourceAndRestart(e.dataTransfer.files[0]);
    return false;
});

// Flow control
function setSourceAndRestart(source) {
    source = source instanceof File ? URL.createObjectURL(source) : source
    img.src = source;
    img.onload = function () {
        let width = img.naturalWidth, height = img.naturalHeight;
        const fixed_size = Math.min(800, Math.max(width, height));
        if (width >= height) {
            width = fixed_size;
            height = Math.round(fixed_size * img.naturalHeight / img.naturalWidth);
        } else {
            height = fixed_size;
            width = Math.round(fixed_size * img.naturalWidth / img.naturalHeight);
        }
        svg.setAttribute('viewBox', `0 0 ${width} ${height}`);
        if (height > width) {
            document.getElementById('canvas-container').style.width = '50%';
            document.getElementById('canvas-container').style.marginBottom = (height / width * 50) + '%';
        } else {
            document.getElementById('canvas-container').style.width = '';
            document.getElementById('canvas-container').style.marginBottom = (height / width * 100) + '%';
        }
        canvas.width = width;
        canvas.height = height;
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
        ctx.getImageData(0, 0, canvas.width, canvas.height);
        restart();
    }
    // Show display
    canvas.style.display = 'block';
    svg.style.display = 'block';
    // Hide upload text
    droptext.style.display = 'none';
}

function restart() {
    if (!img.src) {
        return;
    }
    if (runner) {
        runner.stop();
    }
    clearSvg(svg);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    document.body.id == 'simplification' ? simplification.prepareCanvas(ctx) :
    document.body.id == 'segmentation' ? segmentation.prepareCanvas(ctx) :
    null;
    ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    ctx.filter = 'none';
    progress.value = 0;
    progressregion.style.display = 'block';
    runner = new Runner(getParams());
    runner.run();
}

function clearSvg(svg) {
    while (svg.firstChild) {
        svg.removeChild(svg.firstChild);
    }
}

function moveElements(from, to) {
    while (from.firstChild) {
        to.appendChild(from.removeChild(from.firstChild));
    }
}

function reconfig() {
    if (!runner) {
        return;
    }
    clearSvg(svg);
    runner.reconfig(getParams());
    runner.run();
}

function getParams() {
    return Object.assign({
        'canvas_id': canvas.id,
        'svg_id': svg.id,
    }, global);
}

class Runner {
    constructor (params) {
        this.runner = (
            document.body.id == 'simplification' ? Simplification :
            document.body.id == 'segmentation' ? Segmentation :
        null).new_with_string(JSON.stringify(params));
        this.runner.init();
    }

    run () {
        const This = this;
        let lastRender = performance.now();
        This.timer = setTimeout(function tick () {
            let done = false;
            const startTick = performance.now();
            while (!(done = This.runner.tick()) &&
                performance.now() - startTick < 25) {
            }
            progress.value = This.runner.progress();
            if (progress.value >= progress.max) {
                progressregion.style.display = 'none';
                progress.value = 0;
            }
            if (!done) {
                This.timer = setTimeout(tick, 1);
            }
        }, 1);
    }

    stop () {
        clearTimeout(this.timer);
        this.runner.free();
    }

    reconfig (params) {
        clearTimeout(this.timer);
        this.runner.reconfig(JSON.stringify(params));
    }
}
