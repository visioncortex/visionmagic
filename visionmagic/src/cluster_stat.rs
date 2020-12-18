//! Processor to calculate color statistic on set of clusters
use visioncortex::{ColorStat, ColorStatBuilder};
use visioncortex::color_clusters::Clusters;
use crate::pipeline::Processor as ProcessorTrait;

#[derive(Default)]
pub struct Processor {
    params: Params,
    clusters: Option<Input>,
    builder: ColorStatBuilder,
    counter: usize,
}

/// [`Clusters`]
pub type Input = Clusters;

/// [`ColorStat`]
pub type Output = ColorStat;

#[derive(Default)]
pub struct Params;

impl ProcessorTrait for Processor {

    type Input = Input;
    type Output = Output;
    type Params = Params;

    fn new() -> Self {
        Self::default()
    }

    fn config(&mut self, params: Params) -> bool {
        self.params = params;
        true
    }

    fn input(&mut self, input: Input) -> bool {
        self.clusters = Some(input);
        self.builder = ColorStatBuilder::new();
        let len = self.clusters.as_ref().unwrap().output_len();
        self.counter = if len > 0 { len - 1 } else { 0 };
        len > 0
    }

    fn tick(&mut self) -> bool {
        let view = self.clusters.as_ref().unwrap().view();
        let index = view.clusters_output[self.counter];
        let cluster = view.get_cluster(index);
        self.builder.add(cluster.residue_color());
        if self.counter > 0 {
            self.counter -= 1;
            false
        } else {
            true
        }
    }

    fn progress(&self) -> u32 {
        let total = self.clusters.as_ref().unwrap().output_len() - 1;
        100 - 100 * self.counter as u32 / total as u32
    }

    fn output(&mut self) -> Output {
        let output = self.builder.build();
        log::info!("mean = {:?}, deviation = {:?}", output.mean, output.deviation);
        output
    }

}

impl Processor {
    pub fn take(&mut self) -> Clusters {
        self.clusters.take().unwrap()
    }
}