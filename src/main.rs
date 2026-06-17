#![windows_subsystem = "windows"]

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
        title: "Molestus";
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
            Some(HWND_TOPMOST),
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
    main_window.show().unwrap();

    #[cfg(target_os = "windows")] 
    { 
        std::thread::spawn(move || {
            use std::ffi::CString;
            use windows::Win32::UI::WindowsAndMessaging::FindWindowA;
            let title = CString::new("Molestus").unwrap();
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                unsafe {
                    let hwnd_raw = FindWindowA(None, windows::core::PCSTR(title.as_ptr() as _)).unwrap_or_default();
                    if hwnd_raw.0 != std::ptr::null_mut() {
                        make_window_clickthrough_and_topmost(hwnd_raw.0 as isize);
                        break;
                    }
                }
            }
        });
    }

    let physics_state = Arc::new(Mutex::new(PhysicsState::new()));
    let target_coord = Arc::new(Mutex::new(None::<(f32, f32)>));

    // Physics + Rendering Loop
    let physics_clone = physics_state.clone();
    let window_handle = main_window.as_weak();
    
    std::thread::spawn(move || {
        let mut pixmap = Pixmap::new(1920, 1080).unwrap(); 
        let mut paint = Paint::default(); 
        paint.set_color_rgba8(50, 150, 255, 200); // Blue blob, slightly transparent
        paint.anti_alias = true; 
        
        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = 4.0;
        stroke.line_cap = tiny_skia::LineCap::Round;
        stroke.line_join = tiny_skia::LineJoin::Round;
        let mut stroke_paint = Paint::default();
        stroke_paint.set_color_rgba8(0, 0, 0, 255);
        stroke_paint.anti_alias = true;
        
        loop {
            let mut positions = Vec::new();
            let mut center_pos = (0.0, 0.0);
            {
                let mut state = physics_clone.lock().unwrap();
                state.step();
                
                if let Some(rb) = state.rigid_body_set.get(state.center_handle) {
                    let pos = rb.translation();
                    center_pos = (pos.x, pos.y);
                }
                
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
                    pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
                } 
            } 
            
            // Draw Face dynamically mapped to nodes so it bends automatically!
            if positions.len() == 64 {
                let mut face_paint = Paint::default(); 
                face_paint.set_color_rgba8(0, 0, 0, 255); // Black 
                face_paint.anti_alias = true; 
                
                // Eyes ":"
                let eye1 = (
                    center_pos.0 + 0.4 * (positions[40].0 - center_pos.0),
                    center_pos.1 + 0.4 * (positions[40].1 - center_pos.1)
                );
                let eye2 = (
                    center_pos.0 + 0.4 * (positions[24].0 - center_pos.0),
                    center_pos.1 + 0.4 * (positions[24].1 - center_pos.1)
                );

                let mut pb = PathBuilder::new(); 
                pb.push_circle(eye1.0, eye1.1, 7.0); 
                pb.push_circle(eye2.0, eye2.1, 7.0); 
                if let Some(path) = pb.finish() { 
                    pixmap.fill_path(&path, &face_paint, tiny_skia::FillRule::Winding, Transform::identity(), None); 
                }
                
                // Mouth "D"
                let d_top = (
                    center_pos.0 + 0.25 * (positions[48].0 - center_pos.0),
                    center_pos.1 + 0.25 * (positions[48].1 - center_pos.1)
                );
                let d_bottom = (
                    center_pos.0 + 0.25 * (positions[16].0 - center_pos.0),
                    center_pos.1 + 0.25 * (positions[16].1 - center_pos.1)
                );
                let ctrl1 = ( // bottom-right
                    center_pos.0 + 0.5 * (positions[8].0 - center_pos.0),
                    center_pos.1 + 0.5 * (positions[8].1 - center_pos.1)
                );
                let ctrl2 = ( // top-right
                    center_pos.0 + 0.5 * (positions[56].0 - center_pos.0),
                    center_pos.1 + 0.5 * (positions[56].1 - center_pos.1)
                );

                let mut pb_d = PathBuilder::new();
                pb_d.move_to(d_top.0, d_top.1);
                pb_d.line_to(d_bottom.0, d_bottom.1);
                pb_d.cubic_to(ctrl1.0, ctrl1.1, ctrl2.0, ctrl2.1, d_top.0, d_top.1);
                pb_d.close();

                let mut d_stroke = tiny_skia::Stroke::default();
                d_stroke.width = 10.0;
                d_stroke.line_cap = tiny_skia::LineCap::Round;
                d_stroke.line_join = tiny_skia::LineJoin::Round;

                if let Some(path) = pb_d.finish() { 
                    pixmap.stroke_path(&path, &face_paint, &d_stroke, Transform::identity(), None); 
                } 
            }
            
            let buffer = SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                pixmap.data(),
                pixmap.width(),
                pixmap.height(),
            );
            
            let window_handle_clone = window_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = window_handle_clone.upgrade() {
                    let img = Image::from_rgba8(buffer);
                    w.set_blob_image(img);
                }
            });
            
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
                            rb.apply_impulse((dir.normalize() * 50000.0).into(), true);
                        } else {
                            let mut rng = rand::rng();
                            let mut handles = state.outer_handles.clone();
                            handles.shuffle(&mut rng);
                            for i in 0..5 {
                                if let Some(orb) = state.rigid_body_set.get_mut(handles[i]) {
                                    let opos = orb.translation();
                                    let odir = vector![opos.x - pos.x, opos.y - pos.y];
                                    orb.apply_impulse((odir.normalize() * 10000.0).into(), true);
                                }
                            }
                        }
                    }
                }
            }
            sleep(Duration::from_secs(5)).await; 
        }
    });

    main_window.run()?;
    Ok(())
}
