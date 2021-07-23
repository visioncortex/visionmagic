use wasm_bindgen::prelude::*;
use visionmagic::visioncortex::ColorImage;
use visionmagic::fmm::{painter::Painter, smoother::Smoother};
use std::{u32, u8};

use crate::{canvas::*};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RepairParams {
    pub frame_id: String,
    pub mask_id: String,
    pub blurriness: u32,
}

#[wasm_bindgen]
pub struct Repair {
    frame: Canvas,
    mask: Canvas,
    image_frame: ColorImage,
    image_mask: ColorImage,
    buf_mask: Vec<u8>,
    painter: Painter,
    blurriness: u32,
}

impl Repair {
    pub fn new(params: RepairParams) -> Self {
        let frame = Canvas::new_from_id(&params.frame_id);
        let mask = Canvas::new_from_id(&params.mask_id);
        let blurriness = params.blurriness;
        let image_frame = ColorImage::new();
        let image_mask = ColorImage::new();
        let buf_frame = vec![0; frame.get_image_data_as_color_image(0,0, frame.width() as u32, frame.height() as u32).pixels.len() * 3 / 4];
        let buf_mask = vec![0; mask.get_image_data_as_color_image(0,0, mask.width() as u32, mask.height() as u32).pixels.len()];
        let painter = Painter::new(buf_frame, &buf_mask, image_frame.width as u32, image_frame.height as u32);
        Self {
            frame,
            mask,
            image_frame,
            image_mask,
            buf_mask,
            painter,
            blurriness,
        }
    }
}

#[wasm_bindgen]
impl Repair {
    pub fn new_with_string(params: String) -> Self {
        let params: RepairParams = serde_json::from_str(params.as_str()).unwrap();
        Self::new(params)
    }

    pub fn init(&mut self) {
        self.prepare_image();
    }

    pub fn tick(&mut self) -> bool {
        let mut temp_paint = std::mem::take(&mut self.painter);
        temp_paint = temp_paint.paint();
        let result = Smoother::new( temp_paint.im.buf.clone(), self.image_frame.width as u32, self.image_frame.height as u32, self.blurriness).smooth(&self.buf_mask);
        let mut final_result = ColorImage {
            pixels: self.rgb_to_rgba(&result.im.buf),
            width: result.im.width as usize,
            height: result.im.height as usize,
        };
        self.frame.render_color_image(&mut final_result, 0, 0);
        self.painter = temp_paint;
        self.painter.progress == 100
    }

    pub fn progress(&self) -> u32 {
        self.painter.progress
    }

    fn prepare_image(&mut self) {
        self.image_frame = self.get_image_from_frame();
        self.image_mask = self.get_image_from_mask();
        
        let buf_frame = self.rgba_to_rgb();
        self.buf_mask = self.create_mask_rgba();

        self.painter = Painter::new(buf_frame, &self.buf_mask, self.image_frame.width as u32, self.image_frame.height as u32);
    }

    fn get_image_from_frame(&self) -> ColorImage {
        let width = self.frame.width() as u32;
        let height = self.frame.height() as u32;
        self.frame.get_image_data_as_color_image(0, 0, width, height)
    }

    fn get_image_from_mask(&self) -> ColorImage {
        let width = self.mask.width() as u32;
        let height = self.mask.height() as u32;
        self.mask.get_image_data_as_color_image(0, 0, width, height)
    }

    fn rgb_to_rgba(&mut self, pixels: &Vec<u8>) -> Vec<u8> { // 24 bits -> 32 bits
        let mut result: Vec<u8> = Vec::with_capacity(self.buf_mask.len());
        let length_24 = self.buf_mask.len() * 3 / 4;
        for i in 0..length_24 {
            if result.len() >= self.buf_mask.len() {
                break;
            }
            if i == 0 || i % 3 != 0 {
                result.push(pixels[i]);
            } else {
                result.push(255);
                result.push(pixels[i]);
            }
        }
        result.push(255);
        result
    }

    fn rgba_to_rgb(&mut self) -> Vec<u8> {
        let length_24 = self.image_frame.pixels.len() * 3 / 4;
        let mut result: Vec<u8> = Vec::with_capacity(length_24);
        for i in 0..self.image_frame.pixels.len() {
            if result.len() >= length_24 {
                break;
            }
            if i == 0 || (i + 1) % 4 != 0 {
                result.push(self.image_frame.pixels[i]);
            }
        }
        result
    }

    fn create_mask_rgba(&mut self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::with_capacity(self.buf_mask.len());
        for i in 0..self.image_mask.pixels.len() {
            if result.len() >= self.buf_mask.len() {
                break;
            }
            result.push(self.image_mask.pixels[i]);
        }
        result
    }
}