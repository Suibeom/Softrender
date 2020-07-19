use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use std::time::SystemTime;

mod common;
mod flat;
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
    let tris = varvar::make_triangle_partition(-2.0, 2.0, -1.5, 2.5, 2,2, 0.01);
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
        let (x, y) =elapsed.sin_cos();
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
        let mut pixels = [0u8; 4 * W * H];
        let mut plot = |x: usize, y: usize, color: Pt3| {
            let idx = 4 * x + 4 * W * y;
            let bytes = [0u8, color.x as u8, color.y as u8, color.z as u8];
            if pixels[idx] as u32
                + pixels[idx + 1] as u32
                + pixels[idx + 2] as u32
                + pixels[idx + 3] as u32
                != 0
            {
                pixels[idx..idx + 4].clone_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
                return;
            }
            pixels[idx..idx + 4].clone_from_slice(&bytes);
        };

        // let t1 = varvar::Triangle {
        //     points: [
        //         varvar::Vertex::<Pt3> {
        //             spatial: Pt3 {
        //                 x: 0.1,
        //                 y: 0.0,
        //                 z: 1.0,
        //             },
        //             variable: Pt3 {
        //                 x: 128.0,
        //                 y: 128.0,
        //                 z: 128.0,
        //             },
        //         },
        //         varvar::Vertex::<Pt3> {
        //             spatial: Pt3 {
        //                 x: 1.0,
        //                 y: 0.1,
        //                 z: 1.0,
        //             },
        //             variable: Pt3 {
        //                 x: 128.0,
        //                 y: 128.0,
        //                 z: 128.0,
        //             },
        //         },
        //         varvar::Vertex::<Pt3> {
        //             spatial: Pt3 {
        //                 x: 1.0,
        //                 y: 1.0,
        //                 z: 1.1,
        //             },
        //             variable: Pt3 {
        //                 x: 128.0,
        //                 y: 128.0,
        //                 z: 128.0,
        //             },
        //         },
        //     ],
        // };
        // varvar::draw_triangle(
        //     varvar::prep_triangle(t1, &varvar::DEFAULT_VIEWPORT, &proj),
        //     &mut plot,
        // );

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
