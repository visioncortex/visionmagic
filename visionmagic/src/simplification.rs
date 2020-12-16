use visioncortex::{Color, CompoundPath, PathSimplifyMode};
use visioncortex::color_clusters::{Cluster, Clusters, ClustersView};
use crate::pipeline::Processor as ProcessorTrait;

#[derive(Default)]
pub struct Processor {
    params: Params,
    clusters: Option<Input>,
    buffer: Output,
    counter: usize,
    stop: usize,
}

pub type Input = Clusters;

pub type Output = Vec<OutputUnit>;

pub struct OutputUnit {
    pub path: CompoundPath,
    pub color: Color,
}

pub struct Params {
    pub fidelity: u32,
    pub shape_details: u32,
}

impl Params {
    pub const MAX_FIDELITY: u32 = 65535;
    pub const MAX_SHAPE_DETAILS: u32 = 65535;
}

impl Default for Params {
    /// the default setting makes no simplification
    fn default() -> Self {
        Self {
            fidelity: Self::MAX_FIDELITY,
            shape_details: Self::MAX_SHAPE_DETAILS,
        }
    }
}

impl ProcessorTrait for Processor {

    type Input = Input;
    type Output = Output;
    type Params = Params;

    fn new() -> Self {
        Self::default()
    }

    /// configure simplification parameters; can be reconfigured on runtime
    fn config(&mut self, params: Params) -> bool {
        self.params = params;
        if self.clusters.is_some() {
            self.output();
            let clusters = self.clusters.take().unwrap();
            self.input(clusters);
        }
        true
    }

    fn input(&mut self, input: Input) -> bool {
        self.clusters = Some(input);
        let len = self.clusters.as_ref().unwrap().output_len();
        self.counter = if len > 0 { len - 1 } else { 0 };
        let fraction = self.params.fidelity as f64 / Params::MAX_FIDELITY as f64;
        self.stop = self.counter as usize - (self.counter as f64 * fraction.powi(3)) as usize;
        len > 0
    }

    fn tick(&mut self) -> bool {
        let view = self.clusters.as_ref().unwrap().view();
        let cluster = view.get_cluster(view.clusters_output[self.counter]);
        if let Some(output) = self.process_cluster(&view, cluster) {
            self.buffer.push(output);
        }
        if self.counter > self.stop {
            self.counter -= 1;
            false
        } else {
            true
        }
    }

    fn progress(&self) -> u32 {
        let total = self.clusters.as_ref().unwrap().output_len() - 1;
        if total == self.stop {
            100
        } else {
            100 - 100 * (self.counter - self.stop) as u32 / (total - self.stop) as u32
        }
    }

    /// buffered output;
    /// can be called after each tick or when process ends; each call clears the buffer
    fn output(&mut self) -> Output {
        std::mem::take(&mut self.buffer)
    }

}

impl Processor {
    fn process_cluster(&self, view: &ClustersView, cluster: &Cluster) -> Option<OutputUnit> {
        let path = cluster.to_compound_path(
            &view, false, PathSimplifyMode::None,
            0.0, 0.0, 0, 0.0
        );
        let ratio = 1.0 - self.params.shape_details as f64 / Params::MAX_SHAPE_DETAILS as f64;
        let expr = |x: f64| ((11.0_f64 * x).exp() - 1.0) / ((11.0_f64).exp() - 1.0);
        let max = |a: f64, b: f64| if a > b { a } else { b };
        let threshold = 1.0 + ((view.width * view.height) as f64).sqrt() * expr(ratio);
        if  cluster.rect.width() >= threshold as i32 || 
            cluster.rect.height() >= threshold as i32 {
            // reduction threshold should grow very slowly at first but exponentially later
            let simplified = path.reduce(threshold);
            // patches should be more rounded at higher ratio
            let corner_threshold = interp(ratio, 0.0, 0.75, 0.25 * std::f64::consts::PI, std::f64::consts::PI);
            // patches should expand more at higher ratio
            let outset_ratio = interp(ratio, 0.0, 0.75, 8.0, 4.0);
            let smoothed = simplified.smooth(corner_threshold, outset_ratio, max(4.0, threshold * 0.5));

            Some(OutputUnit {
                path: smoothed,
                color: cluster.residue_color(),
            })
        } else {
            None
        }
    }

    pub fn get_background(&self) -> (Color, Color) {
        let (mut background, mut midground) = (Color::default(), Color::default());
        let view = self.clusters.as_ref().unwrap().view();
        let len = view.clusters_output.len();
        if len >= 1 {
            background = view.get_cluster(view.clusters_output[len - 1]).residue_color();
        }
        if len >= 2 {
            midground = view.get_cluster(view.clusters_output[len - 2]).residue_color();
        }
        (background, midground)
    }
}

fn interp(x: f64, x0: f64, x1: f64, y0: f64, y1: f64) -> f64 {
    if x >= x1 {
        return y1;
    }
    (y0 * (x1 - x) + y1 * (x - x0)) / (x1 - x0)
}