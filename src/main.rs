use slint::{Image, SharedPixelBuffer};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrA, SetWindowLongPtrA, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOMOVE,
    SWP_NOSIZE, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};
use tiny_skia::{Paint, PathBuilder, Pixmap, Transform, Color};
use rapier2d::prelude::*;
use rand::seq::SliceRandom;

mod physics;
mod capture;
mod ai;

use physics::PhysicsState;
use ai::AiModel;

slint::slint! {
    export component MainWindow inherits Window {
        width: 1920px;
        height: 1080px;
        always-on-top: true;
        no-frame: true;
        background: transparent;
        
        in property <image> blob_image;
        
        Image {
            width: 100%;
            height: 100%;
            source: blob_image;
        }
    }
}

// Helper to make it click through
#[cfg(target_os = "windows")]
fn make_window_clickthrough_and_topmost(handle: isize) {
    let hwnd = HWND(handle as _);
    unsafe {
        let ex_style = GetWindowLongPtrA(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrA(
            hwnd,
            GWL_EXSTYLE,
            ex_style | (WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0 | WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0) as isize,
        );
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_window = MainWindow::new()?;

    // Need raw-window-handle feature for slint
    use raw_window_handle::HasRawWindowHandle;
    match main_window.window().raw_window_handle() {
        raw_window_handle::RawWindowHandle::Win32(handle) => {
            make_window_clickthrough_and_topmost(handle.hwnd as isize);
        }
        _ => {}
    }

    let physics_state = Arc::new(Mutex::new(PhysicsState::new()));
    let target_coord = Arc::new(Mutex::new(None::<(f32, f32)>));

    // Physics + Rendering Loop
    let physics_clone = physics_state.clone();
    let window_handle = main_window.as_weak();
    
    std::thread::spawn(move || {
        let mut pixmap = Pixmap::new(1920, 1080).unwrap();
        let mut paint = Paint::default();
        paint.set_color_rgba8(50, 150, 255, 200); // Blue blob
        paint.anti_alias = true;
        
        loop {
            let mut positions = Vec::new();
            {
                let mut state = physics_clone.lock().unwrap();
                state.step();
                
                // Get positions of outer nodes
                for h in &state.outer_handles {
                    if let Some(rb) = state.rigid_body_set.get(*h) {
                        let pos = rb.translation();
                        positions.push((pos.x, pos.y));
                    }
                }
            }
            
            // Draw
            pixmap.fill(Color::TRANSPARENT);
            if positions.len() > 0 {
                let mut pb = PathBuilder::new();
                pb.move_to(positions[0].0, positions[0].1);
                for i in 1..positions.len() {
                    pb.line_to(positions[i].0, positions[i].1);
                }
                pb.close();
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }
            
            let buffer = SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                pixmap.data(),
                pixmap.width(),
                pixmap.height(),
            );
            
            let img = Image::from_rgba8(buffer);
            
            if let Some(w) = window_handle.upgrade() {
                let img_clone = img.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    w.set_blob_image(img_clone);
                });
            }
            
            std::thread::sleep(Duration::from_millis(16));
        }
    });

    // AI Loop
    let target_clone = target_coord.clone();
    let physics_clone2 = physics_state.clone();
    tokio::spawn(async move {
        let ai = AiModel::new(
            "D:\\Molestus\\models\\Qwen3-VL-2B-Instruct-1M-UD-Q6_K_XL.gguf".into(),
            "D:\\Molestus\\models\\mmproj-F16.gguf".into(),
        );
        
        loop {
            if let Ok(img) = capture::capture_primary_monitor() {
                if let Some((tx, ty)) = ai.infer_focus(&img) {
                    *target_clone.lock().unwrap() = Some((tx, ty));
                    
                    let mut state = physics_clone2.lock().unwrap();
                    let center_h = state.center_handle;
                    if let Some(rb) = state.rigid_body_set.get_mut(center_h) {
                        let pos = rb.translation();
                        let dir = vector![tx - pos.x, ty - pos.y];
                        let dist = dir.magnitude();
                        if dist > 10.0 {
                            rb.apply_impulse(dir.normalize() * 50000.0, true);
                        } else {
                            let mut rng = rand::thread_rng();
                            let mut handles = state.outer_handles.clone();
                            handles.shuffle(&mut rng);
                            for i in 0..5 {
                                if let Some(orb) = state.rigid_body_set.get_mut(handles[i]) {
                                    let opos = orb.translation();
                                    let odir = vector![opos.x - pos.x, opos.y - pos.y];
                                    orb.apply_impulse(odir.normalize() * 10000.0, true);
                                }
                            }
                        }
                    }
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    main_window.run()?;
    Ok(())
}
