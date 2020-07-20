use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use std::time::SystemTime;

mod common;
mod varvar;

use crate::common::*;

const W: usize = 320;
const H: usize = 240;

fn main() -> Result<(), String> {
    println!("Hello, world!");
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 320, 240)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, 320, 240)
        .map_err(|e| e.to_string())?;
    // Create a red-green gradient
    let sys_time = SystemTime::now();
    let tris = varvar::make_triangle_partition(-2.5, 2.5, -2.5, 2.5, 15, 15, 0.2);
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        let elapsed = sys_time.elapsed().unwrap().as_secs_f64();
        let (x, y) = elapsed.sin_cos();
        let proj = ProjectionData {
            origin_pt: common::Pt3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            plane_unit_normal: Pt3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            plane_basis_x: Pt3 { x: x, y: y, z: 0.0 },
            plane_basis_y: Pt3 {
                x: y,
                y: -x,
                z: 0.0,
            },
        };
        let mut z_buff = [std::f64::NEG_INFINITY; 4 * W * H];
        let mut pixels = [0u8; 4 * W * H];
        let mut plot = |x: usize, y: usize, z: f64, color: Pt3| {
            let idx = 4 * x + 4 * W * y;
            let bytes = [color.x as u8, color.y as u8, color.z as u8, 0u8];
            if z_buff[idx] < z {
                pixels[idx..idx + 4].clone_from_slice(&bytes);
                z_buff[idx] = z;
            }
        };

        for tri in &tris {
            varvar::draw_triangle(
                varvar::prep_triangle(*tri, &varvar::DEFAULT_VIEWPORT, &proj),
                &mut plot,
            );
        }

        texture
            .update(None, &pixels, 4 * W)
            .map_err(|e| e.to_string())?;

        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
    }

    Ok(())
}
