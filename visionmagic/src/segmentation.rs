//! Processor to group clusters together by the disjoint set algorithm
use std::collections::HashMap;
use visioncortex::{Color, ColorImage, ColorSum};
use visioncortex::color_clusters::{Cluster, Clusters, ClusterIndex};
use visioncortex::disjoint_sets::Forests;
use crate::pipeline::Processor as ProcessorTrait;

#[derive(Default)]
pub struct Processor {
    params: Params,
    clusters: Option<Input>,
    forests: Forests<ClusterIndex>,
    counter: usize,
}

/// [`Clusters`]
pub type Input = Clusters;

/// [`ColorImage`]
pub type Output = ColorImage;

#[derive(Default)]
pub struct Params {
    /// Allowed color difference between shapes in same set
    pub deviation: f64,
}

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
        self.forests = Forests::new();
        for index in view.clusters_output.iter() {
            self.forests.make_set(*index);
        }
        len > 0
    }

    fn tick(&mut self) -> bool {
        let view = self.clusters.as_ref().unwrap().view();
        let myselfi = view.clusters_output[self.counter];
        let myself = view.get_cluster(myselfi);
        let mut votes: Vec<(ClusterIndex, i32)> = myself.neighbours(&view).iter().map(|otheri| {
            let other = view.get_cluster(*otheri);
            (*otheri, Self::color_distance(myself, other))
        }).collect();
        votes.sort_by_key(|v| v.1);
        for (i, v) in votes.iter().enumerate() {
            let diff = v.1 as f64 / 10000.0;
            if  i == 0 && diff > self.params.deviation ||
                i == 1 && diff > self.params.deviation * 0.5 ||
                diff > self.params.deviation * 0.25 {
                break;
            }
            self.forests.union(&myselfi, &v.0);
        }
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
        let mut aggregate = HashMap::new();
        let mut pairs = Vec::new();
        let view = self.clusters.as_ref().unwrap().view();
        for index in view.clusters_output.iter() {
            let label = self.forests.find_set(&index).unwrap();
            pairs.push((*index, label));
            let cluster = view.get_cluster(*index);
            (*aggregate.entry(label).or_insert_with(ColorSum::new)).add(&cluster.residue_color());
        }
        let mut image = ColorImage::new_w_h(view.width as usize, view.height as usize);
        for (index, label) in pairs.iter() {
            let cluster = view.get_cluster(*index);
            let color = aggregate.get(label).unwrap().average();
            cluster.render_to_color_image_with_color(&view, &mut image, &color);
        }
        image
    }

}

impl Processor {
    fn color_distance(myself: &Cluster, other: &Cluster) -> i32 {
        let mycolor = myself.residue_color();
        let otcolor = other.residue_color();
        (10000.0 * Self::color_diff_hsv(mycolor, otcolor)) as i32
    }

    fn color_diff_hsv(a: Color, b: Color) -> f64 {
        let a = a.to_hsv();
        let b = b.to_hsv();
        return 2.0 * wrap(a.h, b.h) + (a.s - b.s).abs() + (a.v - b.v).abs();

        fn wrap(x: f64, y: f64) -> f64 {
            let d = (x - y).abs();
            if d < 0.5 {
                d
            } else {
                1.0 - d
            }
        }
    }
}