import { Simplification, Segmentation } from 'visionmagic';

export function main() {

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

document.addEventListener('paste', function (e) {
    if (e.clipboardData) {
        let items = e.clipboardData.items;
        if (!items) return;

        //access data directly
        for (let i = 0; i < items.length; i++) {
            if (items[i].type.indexOf("image") !== -1) {
                //image
                let blob = items[i].getAsFile();
                let URLObj = window.URL || window.webkitURL;
                let source = URLObj.createObjectURL(blob);
                setSourceAndRestart(source);
            }
        }
        e.preventDefault();
    }
});

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

if (document.getElementById('export-svg')) {
    document.getElementById('export-svg').addEventListener('click', function (e) {
        const blob = new Blob([new XMLSerializer().serializeToString(svg)], {type: 'octet/stream'}),
        url = window.URL.createObjectURL(blob);

        this.href = url;
        this.target = '_blank';

        this.download = 'visionmagic-' + new Date().toISOString().slice(0, 19).replace(/:/g, '').replace('T', ' ') + '.svg';
    });
}

let global = prepareGlobal();

let presetConfigs = preparePresetConfigs();

setup(global, reconfig, restart);

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
    prepareCanvas(ctx);
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

    reconfig (params) {
        clearTimeout(this.timer);
        this.runner.reconfig(JSON.stringify(params));
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

function prepareGlobal() {
    if (document.body.id == 'simplification') {
        return {
            shape_details: parseInt(document.getElementById('shape_details').value),
            fidelity: parseInt(document.getElementById('fidelity').value),
            color_levels: getColorLevels(document.getElementById('color_levels')),
        }
    } else if (document.body.id == 'segmentation') {
        return {
            deviation: 0.5,
            min_size: 64 * 64,
        }
    }
}

function preparePresetConfigs() {
    if (document.body.id == 'simplification') {
        return [
            {
                src: 'assets/samples/luca-bravo-zAjdgNXsMeg-unsplash-s.jpg',
                source: 'https://unsplash.com/photos/zAjdgNXsMeg',
                credit: '<span>Photo by <a href="https://unsplash.com/@lucabravo?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Luca Bravo</a> on <a href="https://unsplash.com/s/photos/landscape?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
            },
            {
                src: 'assets/samples/mark-denton-XH3_OXU3lMk-unsplash-s.jpg',
                source: 'https://unsplash.com/photos/XH3_OXU3lMk',
                credit: '<span>Photo by <a href="https://unsplash.com/@mkd_ie?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Mark Denton</a> on <a href="https://unsplash.com/s/photos/hong-kong-night?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
            },
            {
                src: 'assets/samples/hernan-lucio-gJFvHkUHdSI-unsplash-s.jpg',
                source: 'https://unsplash.com/photos/gJFvHkUHdSI',
                credit: '<span>Photo by <a href="https://unsplash.com/@hernanlucio?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Hernan Lucio</a> on <a href="https://unsplash.com/?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
            },
            {
                src: 'assets/samples/averie-woodard-4nulm-JUYFo-unsplash-s.jpg',
                source: 'https://unsplash.com/photos/4nulm-JUYFo',
                credit: '<span>Photo by <a href="https://unsplash.com/@averieclaire?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">averie woodard</a> on <a href="https://unsplash.com/s/photos/portrait?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
            },
            {
                src: 'assets/samples/elijah-ekdahl-nt69AC1bSdg-unsplash-s.jpg',
                source: 'https://unsplash.com/photos/nt69AC1bSdg',
                credit: '<span>Photo by <a href="https://unsplash.com/@elijah_ekdahl?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Elijah Ekdahl</a> on <a href="https://unsplash.com/s/photos/pet?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
            },
        ]
    } else if (document.body.id == 'segmentation') {
        return [
            {
                src: 'assets/samples/elijah-ekdahl-nt69AC1bSdg-unsplash-s.jpg',
                source: 'https://unsplash.com/photos/nt69AC1bSdg',
                credit: '<span>Photo by <a href="https://unsplash.com/@elijah_ekdahl?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Elijah Ekdahl</a> on <a href="https://unsplash.com/s/photos/pet?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
            },
            {
                src: 'assets/samples/averie-woodard-4nulm-JUYFo-unsplash-s.jpg',
                source: 'https://unsplash.com/photos/4nulm-JUYFo',
                credit: '<span>Photo by <a href="https://unsplash.com/@averieclaire?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">averie woodard</a> on <a href="https://unsplash.com/s/photos/portrait?utm_source=unsplash&amp;utm_medium=referral&amp;utm_content=creditCopyText">Unsplash</a></span>',
            },
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
}


function setup(params, reconfig, restart) {
    if (document.body.id == 'simplification') {
        for (const parameter of ['shape_details', 'fidelity']) {
            document.getElementById(parameter).addEventListener('input', function (e) {
                params[parameter] = parseInt(this.value);
                document.getElementById(parameter+'_value').innerHTML = this.value;
                reconfig();
            });
        }
    
        document.getElementById('color_levels').addEventListener('change', function (e) {
            params.color_levels = getColorLevels(this);
            document.getElementById('color_levels'+'_value').innerHTML = params.color_levels;
            restart();
        });
    } else if (document.body.id == 'segmentation') {
        displayParams(params);
        document.getElementById('deviation').addEventListener('input', function (e) {
            params.deviation = parseFloat(this.value);
            document.getElementById('deviation'+'_value').innerHTML = displayDeviation(params);
            reconfig();
        });
    
        document.getElementById('min_size').addEventListener('input', function (e) {
            params.min_size = getMinSize(this.value);
            document.getElementById('min_size'+'_value').innerHTML = this.value;
            reconfig();
        });
    }
}

function getColorLevels(el) {
    return Math.max(2, 1 << parseInt(el.value));
}

function displayColorLevels(params) {
    return Math.log2(params.color_levels)
}

function displayParams(params) {
    for (const parameter of ['shape_details', 'fidelity']) {
        document.getElementById(parameter).value = params[parameter];
        document.getElementById(parameter+'_value').innerHTML = params[parameter];
    }

    document.getElementById('color_levels').value = displayColorLevels(params);
    document.getElementById('color_levels'+'_value').innerHTML = params.color_levels;
}


function displayDeviation(params) {
    return Math.round(params.deviation * 100) / 100;
}

function displayMinSize(params) {
    return Math.round(Math.sqrt(params.min_size));
}

function getMinSize(val) {
    return parseInt(val) * parseInt(val);
}

function displayParams(params) {
    if (document.body.id == 'simplification') {
        for (const parameter of ['shape_details', 'fidelity']) {
            document.getElementById(parameter).value = params[parameter];
            document.getElementById(parameter+'_value').innerHTML = params[parameter];
        }
        document.getElementById('color_levels').value = displayColorLevels(params);
        document.getElementById('color_levels'+'_value').innerHTML = params.color_levels;    
    } else if (document.body.id == 'segmentation') {
        document.getElementById('deviation').value = 
        document.getElementById('deviation'+'_value').innerHTML = displayDeviation(params);
    
        document.getElementById('min_size').value = displayMinSize(params);
        document.getElementById('min_size'+'_value').innerHTML = displayMinSize(params);
    }
}

function prepareCanvas(ctx) {
    ctx.filter = 'blur(1px)';
}

}