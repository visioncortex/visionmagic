use wasm_bindgen::prelude::*;
use visionmagic::visioncortex::ColorImage;
use visionmagic::{Processor, Clustering, Segmentation as Segmenter, Aggregation};

use crate::canvas::*;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SegmentationParams {
    pub canvas_id: String,
    pub svg_id: String,
    /// range 0.0~infinity
    pub deviation: f64,
    /// range 0~infinity
    pub min_size: u32,
}

enum Stage {
    New,
    Clustering(Clustering),
    Segmenter(Segmenter),
    Reclustering(Clustering),
    Aggregation(Aggregation),
}

impl Default for Stage {
    /// the default setting makes no simplification
    fn default() -> Self {
        Self::New
    }
}

#[wasm_bindgen]
pub struct Segmentation {
    canvas: Canvas,
    stage: Stage,
    segmenter: Segmenter,
    params: SegmentationParams,
}

impl Segmentation {
    pub fn new(params: SegmentationParams) -> Self {
        let canvas = Canvas::new_from_id(&params.canvas_id);
        Self {
            canvas,
            stage: Stage::New,
            segmenter: Segmenter::new(),
            params,
        }
    }
}

#[wasm_bindgen]
impl Segmentation {

    pub fn new_with_string(params: String) -> Self {
        let params: SegmentationParams = serde_json::from_str(params.as_str()).unwrap();
        Self::new(params)
    }

    pub fn reconfig(&mut self, params: String) {
        self.params = serde_json::from_str(params.as_str()).unwrap();
        let clustering_params = self.clustering_params();
        let segmenter_params = self.segmenter_params();
        match &mut self.stage {
            Stage::New => panic!("uninitialized"),
            Stage::Clustering(clustering) => {
                clustering.config(clustering_params);
            },
            Stage::Segmenter(segmenter) => {
                segmenter.config(segmenter_params);
            },
            Stage::Reclustering(_) | Stage::Aggregation(_) => {
                let mut segmenter = std::mem::take(&mut self.segmenter);
                segmenter.config(segmenter_params);
                self.stage = Stage::Segmenter(segmenter);
            },
        }
    }

    fn clustering_params(&self) -> <Clustering as Processor>::Params {
        type Params = <Clustering as Processor>::Params;
        let mut params = Params::default();
        params.hierarchical = 64;
        params
    }

    fn segmenter_params(&self) -> <Segmenter as Processor>::Params {
        type Params = <Segmenter as Processor>::Params;
        let mut params = Params::default();
        params.deviation = self.params.deviation;
        params
    }

    fn aggregation_params(&self) -> <Aggregation as Processor>::Params {
        type Params = <Aggregation as Processor>::Params;
        let mut params = Params::default();
        params.deviation = self.params.deviation;
        params.min_size = self.params.min_size;
        params
    }

    pub fn init(&mut self) {
        self.prepare_clustering();
    }

    pub fn tick(&mut self) -> bool {
        match &mut self.stage {
            Stage::New => panic!("uninitialized"),
            Stage::Clustering(processor) => {
                if processor.tick() {
                    self.prepare_segmenter();
                }
                false
            },
            Stage::Segmenter(processor) => {
                if processor.tick() {
                    self.prepare_reclustering();
                }
                false
            },
            Stage::Reclustering(processor) => {
                if processor.tick() {
                    self.prepare_aggregation();
                }
                false
            },
            Stage::Aggregation(processor) => {
                if processor.tick() {
                    self.aggregation_output();
                    return true;
                }
                false
            },
        }
    }

    pub fn progress(&self) -> u32 {
        match &self.stage {
            Stage::New => 0,
            Stage::Clustering(processor) => processor.progress(),
            _ => 100,
        }
    }

    fn get_image_from_canvas(&self) -> ColorImage {
        let width = self.canvas.width() as u32;
        let height = self.canvas.height() as u32;
        self.canvas.get_image_data_as_color_image(0, 0, width, height)
    }

    fn prepare_clustering(&mut self) {
        let image = self.get_image_from_canvas();
        let mut clustering = Clustering::new();
        clustering.config(self.clustering_params());
        clustering.input(image);
        self.stage = Stage::Clustering(clustering);
    }

    fn prepare_segmenter(&mut self) {
        let mut segmenter = Segmenter::new();
        let stage = std::mem::take(&mut self.stage);
        if let Stage::Clustering(mut clustering) = stage {
            segmenter.config(self.segmenter_params());
            segmenter.input(clustering.output());
        } else {
            panic!("must be in Stage::Clustering")
        }
        self.stage = Stage::Segmenter(segmenter);
    }

    fn prepare_reclustering(&mut self) {
        type Params = <Clustering as Processor>::Params;
        let mut clustering = Clustering::new();
        let mut params = Params::default();
        params.hierarchical = 64;
        let stage = std::mem::take(&mut self.stage);
        if let Stage::Segmenter(mut segmenter) = stage {
	        clustering.config(params);
	        clustering.input(segmenter.output());
            self.segmenter = segmenter;
        } else {
            panic!("must be in Stage::Segmenter")
        }
        self.stage = Stage::Reclustering(clustering);
    }

    fn prepare_aggregation(&mut self) {
        let mut aggregation = Aggregation::new();
        let stage = std::mem::take(&mut self.stage);
        if let Stage::Reclustering(mut processor) = stage {
            aggregation.config(self.aggregation_params());
            aggregation.input(processor.output());
        } else {
            panic!("must be in Stage::Reclustering")
        }
        self.stage = Stage::Aggregation(aggregation);
    }

    fn aggregation_output(&mut self) {
        if let Stage::Aggregation(processor) = &mut self.stage {
            let mut image = processor.output();
            self.canvas.clear();
            self.canvas.render_color_image(&mut image, 0, 0);
        } else {
            panic!("must be in Stage::Aggregation")
        }
    }

}