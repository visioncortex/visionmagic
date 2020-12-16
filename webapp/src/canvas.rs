use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Path2d, HtmlCanvasElement, ImageData};
use visionmagic::visioncortex::{Color, ColorImage, CompoundPath, PointF64};

use super::common::document;

pub struct Canvas {
    html_canvas: HtmlCanvasElement,
    cctx: CanvasRenderingContext2d,
}

impl Canvas {
    pub fn new_from_id(canvas_id: &str) -> Canvas {
        let html_canvas = document().get_element_by_id(canvas_id).unwrap();
        let html_canvas: HtmlCanvasElement = html_canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let cctx = html_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Canvas {
            html_canvas,
            cctx,
        }
    }

    pub fn clear(&mut self) {
        self.cctx.clear_rect(0.0, 0.0, self.width() as f64, self.height() as f64);
    }

    pub fn width(&self) -> usize {
        self.html_canvas.width() as usize
    }

    pub fn height(&self) -> usize {
        self.html_canvas.height() as usize
    }

    pub fn get_image_data(&self, x: u32, y: u32, width: u32, height: u32) -> Vec<u8> {
        let image = self
            .cctx
            .get_image_data(x as f64, y as f64, width as f64, height as f64)
            .unwrap();
        image.data().to_vec()
    }

    pub fn get_image_data_as_color_image(&self, x: u32, y: u32, width: u32, height: u32) -> ColorImage {
        ColorImage {
            pixels: self.get_image_data(x, y, width, height),
            width: width as usize,
            height: height as usize,
        }
    }

    pub fn render_color_image(&mut self, image: &mut ColorImage, x: u32, y: u32) {
        let image =
            ImageData::new_with_u8_clamped_array_and_sh(
                wasm_bindgen::Clamped(&mut image.pixels), image.width as u32, image.height as u32
            ).unwrap();
        self.cctx.put_image_data(&image, x as f64, y as f64).unwrap();
    }

    pub fn set_fill_style(&mut self, color: &Color) {
        self.cctx.set_fill_style(JsValue::from_str(color.to_color_string().as_ref()).as_ref());
    }

    pub fn fill_path(&mut self, paths: &CompoundPath, color: &Color) {
        let (string, offset) = paths.to_svg_string(true, PointF64::default());
        let path = Path2d::new_with_path_string(string.as_str()).unwrap();
        self.set_fill_style(color);
        self.cctx.reset_transform().unwrap();
        self.cctx.translate(offset.x, offset.y).unwrap();
        self.cctx.fill_with_path_2d(&path);
        self.cctx.reset_transform().unwrap();
    }
}