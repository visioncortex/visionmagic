use wasm_bindgen::prelude::*;
use visionmagic::visioncortex::ColorImage;
use visionmagic::{Processor, Clustering, Simplification as Simplifier};

use crate::canvas::*;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SimplificationParams {
    pub canvas_id: String,
    pub svg_id: String,
    /// range 0~65536
    pub fidelity: u32,
    /// range 0~255
    pub color_levels: u32,
    /// range 1~65536
    pub shape_details: u32,
}

enum Stage {
    New,
    Clustering(Clustering),
    Simplifier(Simplifier),
}

impl Default for Stage {
    /// the default setting makes no simplification
    fn default() -> Self {
        Self::New
    }
}

#[wasm_bindgen]
pub struct Simplification {
    canvas: Canvas,
    stage: Stage,
    params: SimplificationParams,
}

impl Simplification {
    pub fn new(params: SimplificationParams) -> Self {
        let canvas = Canvas::new_from_id(&params.canvas_id);
        Self {
            canvas,
            stage: Stage::New,
            params,
        }
    }
}

#[wasm_bindgen]
impl Simplification {

    pub fn new_with_string(params: String) -> Self {
        let params: SimplificationParams = serde_json::from_str(params.as_str()).unwrap();
        Self::new(params)
    }

    pub fn reconfig(&mut self, params: String) {
        self.params = serde_json::from_str(params.as_str()).unwrap();
        let clustering_params = self.clustering_params();
        let simplifier_params = self.simplifier_params();
        match &mut self.stage {
            Stage::New => panic!("uninitialized"),
            Stage::Clustering(clustering) => {
                clustering.config(clustering_params);
            },
            Stage::Simplifier(simplifier) => {
                simplifier.config(simplifier_params);
            },
        }
    }

    fn clustering_params(&self) -> <Clustering as Processor>::Params {
        type Params = <Clustering as Processor>::Params;
        let mut params = Params::default();
        params.color_levels = self.params.color_levels;
        params
    }

    fn simplifier_params(&self) -> <Simplifier as Processor>::Params {
        type Params = <Simplifier as Processor>::Params;
        Params {
            fidelity: self.params.fidelity,
            shape_details: self.params.shape_details,
        }
    }

    pub fn init(&mut self) {
        self.prepare_clustering();
    }

    pub fn tick(&mut self) -> bool {
        match &mut self.stage {
            Stage::New => panic!("uninitialized"),
            Stage::Clustering(processor) => {
                if processor.tick() {
                    self.prepare_simplify();
                }
                false
            },
            Stage::Simplifier(processor) => {
                if processor.tick() {
                    self.simplifier_output();
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

    fn prepare_simplify(&mut self) {
        let mut simplifier = Simplifier::new();
        simplifier.config(self.simplifier_params());
        let stage = std::mem::take(&mut self.stage);
        if let Stage::Clustering(mut clustering) = stage {
            simplifier.input(clustering.output());
        } else {
            panic!("must be in Stage::Clustering")
        }
        self.stage = Stage::Simplifier(simplifier);
    }

    fn simplifier_output(&mut self) {
        if let Stage::Simplifier(simplifier) = &mut self.stage {
            let shapes = simplifier.output();
            for shape in shapes.iter() {
                self.canvas.fill_path(
                    &shape.path,
                    &shape.color,
                );
            }
        } else {
            panic!("must be in Stage::Simplifier")
        }
    }

}