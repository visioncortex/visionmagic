import { Repair } from 'visionmagic';

export function main() {

let runner;
const canvas = document.getElementById('frame');
const mask = document.getElementById('mask');
const drawingElement = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
const svg = document.getElementById('svg');
const img = new Image();
const progress = document.getElementById('progressbar');
const progressregion = document.getElementById('progressregion');

// Hide canas and svg on load
mask.style.display = 'none';
canvas.style.display = 'none';
svg.style.display = 'none';
if (drawingElement) {
    drawingElement.style.display = 'none';
}

const watermark = { light: new Image(), dark: new Image() };
watermark.light.src = 'assets/visioncortex-logo-watermark-light.png';
watermark.dark.src = 'assets/visioncortex-logo-watermark-dark.png';

let global = {
    brusher: 20,
    shape: 'Circle',
    blurriness: 10,
};

let presetConfigs = preparePresetConfigs();

setup(global);


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

if (document.getElementById('galleryslider')) {
    for (let i = 0; i < presetConfigs.length; i++) {
        document.getElementById('galleryslider').innerHTML += 
        `<li>
        <div class="galleryitem uk-panel uk-flex uk-flex-center">
            <a href="#">
                <img src="${presetConfigs[i].src}" title="${presetConfigs[i].source}">
            </a>
        </div>
        </li>`;
        document.getElementById('credits-modal-content').innerHTML += 
        `<p>${presetConfigs[i].credit}</p>`;
    }
}

// Gallery
let chooseGalleryButtons = document.querySelectorAll('.galleryitem a');
chooseGalleryButtons.forEach(item => {
    item.addEventListener('click', function (e) {
        // Load preset template config
        let i = Array.prototype.indexOf.call(chooseGalleryButtons, item);
        // Load preset parameters
        Object.assign(global, presetConfigs[i]);
        displayParams(global);
        // Set source as specified
        setSourceAndRestart(this.firstElementChild.src);
    });
});

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
        if (document.body.id != 'repair') {
            restart();
        }
    }
    // Show display
    canvas.style.display = 'block';
    svg.style.display = 'block';
    if (drawingElement) {
        drawingElement.style.display = 'block';
        mask.style.display = 'block';
    }
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

function getParams() {
    ctx.drawImage(mask, 0, 0);
    return Object.assign({
        'frame_id': 'frame',
        'mask_id': 'canvas',
    }, global);
}

class Runner {
    constructor (params) {
        this.runner = Repair.new_with_string(JSON.stringify(params));
        this.runner.init();
    }

    run () {
        const This = this;
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
            } else {
                renderWatermark();
            }
        }, 1);
    }

    stop () {
        clearTimeout(this.timer);
        this.runner.free();
    }
}

function renderWatermark() {
    const w = 125 * 1.25, h = 26 * 1.25;
    const imagedata = ctx.getImageData(canvas.width - w, canvas.height - h, w, h);
    let count = 0, sum = 0;
    for (let i=0; i<w*h; i+=3) {
        if (i != 0 && i % 4 == 0) {
            continue;
        }
        count += 1;
        sum += imagedata.data[i];
    }
    const ave = sum / count;
    if ((64 <= ave && ave < 128) || (192 <= ave && ave < 256)) {
        ctx.drawImage(watermark.dark, canvas.width - w, canvas.height - h, w, h);
    } else {
        ctx.drawImage(watermark.light, canvas.width - w, canvas.height - h, w, h);
    }
}

function preparePresetConfigs() {
    return [
        {
            src: 'assets/samples/sanved-bangale-oSj50kWaU6E-unsplash.jpg',
            source: 'https://unsplash.com/photos/oSj50kWaU6E',
            credit: '<span>Photo by <a href="https://unsplash.com/@sanvedbangale23?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Sanved Bangale</a> on <a href="https://unsplash.com/?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
        },
        {
            src: 'assets/samples/ashley-byrd-lywKTBRDV3I-unsplash.jpg',
            source: 'https://unsplash.com/photos/lywKTBRDV3I',
            credit: '<span>Photo by <a href="https://unsplash.com/@byrdman85?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Ashley Byrd</a> on <a href="https://unsplash.com/?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
        },
        {
            src: 'assets/samples/jennifer-griffin-RFP4D5hGTB0-unsplash.jpg',
            source: 'https://unsplash.com/photos/RFP4D5hGTB0',
            credit: '<span>Photo by <a href="https://unsplash.com/@dotjpg?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Jennifer Griffin</a> on <a href="https://unsplash.com/?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
        },
    ]
}

function setup(params) {
    displayParams(params);

    document.getElementById('brusher').addEventListener('input', function (e) {
        params.brusher = parseFloat(this.value);
        document.getElementById('brusher'+'_value').innerHTML = displayBrusherSize(params);
    });
    document.getElementById('blurriness').addEventListener('input', function (e) {
        params.blurriness = parseInt(this.value);
        document.getElementById('blurriness'+'_value').innerHTML = displayblurrinessSize(params);
    });
    
    const svg = document.querySelector('#svg'); // detect
    const frame = document.querySelector('#frame'); // frame
    const frameCtx = frame.getContext('2d');
    const mask = document.querySelector('#mask'); // frame
    const maskCtx = mask.getContext('2d');
    const canvas = document.querySelector('#canvas'); // mask
    const canvasCtx = canvas.getContext('2d');

    let displayRatio = 1;
    let drawingMode = false;
    let cX = 0;
    let cY = 0;

    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    mask.width = window.innerWidth;
    mask.height = window.innerHeight;

    svg.addEventListener('mousemove', (e) => draw(e));
    svg.addEventListener('mousedown', (e) => {
        displayRatio = (frame.clientWidth / frame.width)
        canvas.width = frame.width;
        canvas.height = frame.height;
        mask.width = frame.width;
        mask.height = frame.height;
        drawingMode = true;
        cX = e.offsetX / displayRatio;
        cY = e.offsetY / displayRatio;

        frameCtx.lineWidth = params.brusher;
        canvasCtx.lineWidth = params.brusher + 2;
        maskCtx.lineWidth = params.brusher;
    });
    svg.addEventListener('mouseup', (e) => { drawingMode = false; restart(); });
    svg.addEventListener('mouseout', (e) => { drawingMode = false; });

    function draw(e) {
        if (!drawingMode) return;
        fillCtx(frameCtx, cX, cY, e, displayRatio);
        fillCtx(canvasCtx, cX, cY, e, displayRatio);
        fillCtx(maskCtx, cX, cY, e, displayRatio);
        cX = e.offsetX / displayRatio;
        cY = e.offsetY / displayRatio;
    }
}
function fillCtx(ctx, cX, cY, e, displayRatio) {
    ctx.lineJoin = 'round'; 
    ctx.lineCap = 'round';
    ctx.strokeStyle = '#000';
    ctx.beginPath();
    ctx.globalCompositeOperation = "source-over";
    ctx.moveTo(cX, cY);
    ctx.lineTo(e.offsetX / displayRatio, e.offsetY / displayRatio);
    ctx.stroke();
}

function displayBrusherSize(params) {
    return Math.round(params.brusher * 100) / 100;
}
function displayblurrinessSize(params) {
    return Math.round(params.blurriness);
}

function displayParams(params) {
    document.getElementById('brusher').value = 
    document.getElementById('brusher'+'_value').innerHTML = displayBrusherSize(params);
    document.getElementById('blurriness').value = 
    document.getElementById('blurriness'+'_value').innerHTML = displayblurrinessSize(params);
}

} // end main