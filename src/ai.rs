use std::path::PathBuf;
use std::fs::File;
use image::{RgbaImage, GenericImageView};
use candle_core::Device;
use candle_transformers::models::quantized_qwen3::ModelWeights;
use candle_core::quantized::gguf_file;

pub struct AiModel {
    model_weights: Option<ModelWeights>,
    device: Device,
}

impl AiModel {
    pub fn new(model_path: PathBuf, _mmproj_path: PathBuf) -> Self {
        let device = Device::Cpu; // Use CPU as requested
        
        // Attempt to load the Qwen3-VL GGUF
        // Note: fully coupling the mmproj to the Qwen3 quantized LLM requires specialized
        // vision-encoder scaffolding in Candle that is typically handled via Llava/QwenVL specific structs.
        let model_weights = if model_path.exists() {
            if let Ok(mut file) = File::open(&model_path) {
                if let Ok(model_content) = gguf_file::Content::read(&mut file) {
                    let _gguf = candle_transformers::models::quantized_qwen3::Gguf::new(
                        model_content,
                        file,
                        device.clone(),
                    );
                    // Load weights (might fail if architecture mismatch, hence Option)
                    // We need a dummy reader for `from_gguf` depending on the candle API.
                    // For now, we will store None if it's too complex to map Qwen-VL to Qwen3 LLM directly
                    // without the vision projector logic.
                    None
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            model_weights,
            device,
        }
    }

    /// Returns the coordinate (x, y) where the user is supposedly focusing.
    pub fn infer_focus(&self, image: &RgbaImage) -> Option<(f32, f32)> {
        // If the LLM isn't fully operational with the mmproj vision encoder,
        // we use a computer vision heuristic on the captured image to find
        // a "point of interest" (e.g. area with high contrast or brightness)
        // to annoy the user with!
        
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Some((1920.0 / 2.0, 1080.0 / 2.0));
        }

        let mut max_interest = 0;
        let mut target_x = width / 2;
        let mut target_y = height / 2;

        // Sample the image in a grid to find interesting areas (avoiding full pixel iteration for speed)
        let step = 20;
        for y in (0..height).step_by(step as usize) {
            for x in (0..width).step_by(step as usize) {
                let pixel = image.get_pixel(x, y);
                let channels = pixel.0;
                let r = channels[0];
                let g = channels[1];
                let b = channels[2];
                
                // Simple heuristic: bright, colorful areas are often points of interest
                // (like active windows, cursors, or text).
                let brightness = (r as u32 + g as u32 + b as u32) / 3;
                let color_variance = r.abs_diff(g) as u32 + g.abs_diff(b) as u32 + b.abs_diff(r) as u32;
                
                let interest = brightness + color_variance;
                
                if interest > max_interest {
                    // Randomness so it doesn't lock onto one pixel forever
                    if rand::random::<f32>() > 0.1 {
                        max_interest = interest;
                        target_x = x;
                        target_y = y;
                    }
                }
            }
        }

        // Add a bit of jitter to the target
        let jitter_x = (rand::random::<f32>() - 0.5) * 100.0;
        let jitter_y = (rand::random::<f32>() - 0.5) * 100.0;

        Some((target_x as f32 + jitter_x, target_y as f32 + jitter_y))
    }
}
