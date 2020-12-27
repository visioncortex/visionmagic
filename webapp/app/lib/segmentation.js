export function global() {
    return {
        deviation: 0.5,
        min_size: 64 * 64,
    }
}

export function presetConfigs() {
    return []
}

export function setup(params, reconfig, restart) {

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

function displayDeviation(params) {
    return Math.round(params.deviation * 100) / 100;
}

function displayMinSize(params) {
    return Math.round(Math.sqrt(params.min_size));
}

function getMinSize(val) {
    return parseInt(val) * parseInt(val);
}

export function displayParams(params) {
    document.getElementById('deviation').value = 
    document.getElementById('deviation'+'_value').innerHTML = displayDeviation(params);

    document.getElementById('min_size').value = displayMinSize(params);
    document.getElementById('min_size'+'_value').innerHTML = displayMinSize(params);
}

export function prepareCanvas(ctx) {
    ctx.filter = 'blur(1px)';
}