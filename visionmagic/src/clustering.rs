//! Processor to perform clustering & hierarchical clustering on an image
use visioncortex::ColorImage;
use visioncortex::color_clusters::{Clusters, IncrementalBuilder, Runner, RunnerConfig, HIERARCHICAL_MAX};
use crate::pipeline::Processor as ProcessorTrait;

#[derive(Default)]
pub struct Processor {
    params: Params,
    builder: Option<IncrementalBuilder>,
}

/// [`ColorImage`]
pub type Input = ColorImage;

/// [`Clusters`]
pub type Output = Clusters;

pub struct Params {
    /// Valid range is 1~256. More levels means finer gradient
    pub color_levels: u32,
    /// Perform hierarchical clustering up to this size (area)
    pub hierarchical: u32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            color_levels: Self::MAX_COLOR_LEVELS,
            hierarchical: HIERARCHICAL_MAX,
        }
    }
}

impl Params {
    pub const MAX_COLOR_LEVELS: u32 = 256;
}

impl ProcessorTrait for Processor {

    type Input = Input;
    type Output = Output;
    type Params = Params;

    fn new() -> Self {
        Self::default()
    }

    /// configure clustering parameters; can be reconfigured on runtime
    fn config(&mut self, params: Params) -> bool {
        self.params = params;
        if self.builder.is_some() {
            let clusters = self.output();
            self.input(clusters.take_image());
        }
        true
    }

    fn input(&mut self, input: Input) -> bool {
        let non_empty = input.width > 0 && input.height > 0;
        let runner = Runner::new(RunnerConfig {
            diagonal: false,
            hierarchical: self.params.hierarchical,
            batch_size: 25600,
            good_min_area: 1,
            good_max_area: (input.width * input.height) as usize,
            is_same_color_a: 0,
            is_same_color_b: 1,
            deepen_diff: (Params::MAX_COLOR_LEVELS / self.params.color_levels) as i32,
            hollow_neighbours: 0,
        }, input);
        self.builder = Some(runner.start());
        non_empty
    }

    fn tick(&mut self) -> bool {
        self.builder.as_mut().unwrap().tick()
    }

    fn progress(&self) -> u32 {
        self.builder.as_ref().unwrap().progress()
    }

    /// to be called once only after process ends
    fn output(&mut self) -> Output {
        self.builder.take().unwrap().result()
    }

}
