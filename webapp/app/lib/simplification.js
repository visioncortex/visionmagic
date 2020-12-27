export function global() {
    return {
        shape_details: parseInt(document.getElementById('shape_details').value),
        fidelity: parseInt(document.getElementById('fidelity').value),
        color_levels: getColorLevels(document.getElementById('color_levels')),
    }
}

export function presetConfigs() {
    return []
}

export function setup(params, reconfig, restart) {
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
}

function getColorLevels(el) {
    return Math.max(2, 1 << parseInt(el.value));
}

function displayColorLevels(params) {
    return Math.log2(params.color_levels)
}

export function displayParams(params) {
    for (const parameter of ['shape_details', 'fidelity']) {
        document.getElementById(parameter).value = params[parameter];
        document.getElementById(parameter+'_value').innerHTML = params[parameter];
    }

    document.getElementById('color_levels').value = displayColorLevels(params);
    document.getElementById('color_levels'+'_value').innerHTML = params.color_levels;
}

export function prepareCanvas(ctx) {}