use xcap::Monitor;
use image::{RgbaImage, ImageBuffer, Rgba};
use std::error::Error;

pub fn capture_primary_monitor() -> Result<RgbaImage, Box<dyn Error>> {
    let monitors = Monitor::all()?;
    
    // Find the primary monitor, or just use the first one
    let primary = monitors.into_iter().find(|m| m.is_primary()).unwrap_or_else(|| {
        Monitor::all().unwrap().into_iter().next().unwrap()
    });

    let image = primary.capture_image()?;
    // `image` is an RgbaImage (from the `image` crate)
    Ok(image)
}
