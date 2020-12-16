
use visioncortex::ColorImage;
use visioncortex::color_clusters::{Clusters, ClusterIndex};

use crate::pipeline::Processor as ProcessorTrait;

#[derive(Default)]
pub struct Processor {
    params: Params,
    clusters: Option<Input>,
    image: Output,
    counter: usize,
}

pub type Input = Clusters;

pub type Output = ColorImage;

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
        if self.clusters.is_some() {
            let cluster = self.clusters.take().unwrap();
            self.input(cluster);
        }
        true
    }

    fn input(&mut self, input: Input) -> bool {
        self.clusters = Some(input);
        let view = self.clusters.as_ref().unwrap().view();
        let len = view.clusters_output.len();
        self.counter = if len > 0 { len - 1 } else { 0 };
        self.image = ColorImage::new_w_h(view.width as usize, view.height as usize);
        len > 0
    }

    fn tick(&mut self) -> bool {
        let view = self.clusters.as_ref().unwrap().view();
        let myselfi = view.clusters_output[self.counter];
        let myself = view.get_cluster(myselfi);
        let mut votes: Vec<(ClusterIndex, i32)> = myself.neighbours(&view).iter().map(|otheri| {
            let other = view.get_cluster(*otheri);
            (*otheri, super::Segmentation::color_distance(myself, other))
        }).collect();
        votes.sort_by_key(|v| v.1);
        let color = if votes.len() <= 2 || myself.area() < 1024 {
            view.get_cluster(votes[0].0).residue_color()
        } else {
            myself.residue_color()
        };
        myself.render_to_color_image_with_color(&view, &mut self.image, &color);
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

    /// to be called once only after process ends
    fn output(&mut self) -> Output {
        std::mem::take(&mut self.image)
    }

}
