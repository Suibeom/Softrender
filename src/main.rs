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

const TxH: usize = 256;
const TxW: usize = 256;

fn gen_texture_1() -> [u8; TxH * TxW * 4] {
    let mut tex = [0u8; TxH * TxW * 4];
    for y in 0..TxH {
        for x in 0..TxW {
            let idx = 4 * x + 4 * TxW * y;
            tex[idx] = ((10 * x as i32) % 255) as u8;
            tex[idx + 1] = ((30 * y as i32) % 255) as u8;
            tex[idx + 2] = ((5 * x as i32 & 5 * y as i32) % 255) as u8;
            tex[idx + 3] = ((x as i32 & y as i32) % 255) as u8;
        }
    }
    return tex;
}

fn gen_texture_2() -> [u8; TxH * TxW * 4] {
    let mut tex = [0u8; TxH * TxW * 4];
    for y in 0..TxH {
        for x in 0..TxW {
            let idx = 4 * x + 4 * TxW * y;
            tex[idx] = ((10 * x as i32) % 255) as u8;
            tex[idx + 1] = ((30 * y as i32) % 255) as u8;
            tex[idx + 2] = ((5 * x as i32 & 5 * y as i32) % 255) as u8;
            tex[idx + 3] = ((x as i32 & y as i32) % 255) as u8;
        }
    }
    return tex;
}

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
    let sys_time = SystemTime::now();
    let tris = varvar::make_triangle_partition(-2.5, 2.5, -2.5, 2.5, 100, 100, 0.1);
    let mut event_pump = sdl_context.event_pump()?;
    let tex = gen_texture_1();
    let uv_to_xy = |x: f64, y: f64| {
        let (x_min, x_max) = (-2.6, 2.6);
        let (y_min, y_max) = (-2.6, 2.6);

        return (
            (((x - x_min) / (x_max - x_min)) * TxW as f64) as usize,
            (((y - y_min) / (y_max - y_min)) * TxH as f64) as usize,
        );
    };

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
        let mut plot = |x: usize, y: usize, z: f64, uv: Pt3| {
            let idx = 4 * x + 4 * W * y;
            let (tex_x, tex_y) = uv_to_xy(uv.x, uv.y);
            let tex_idx = 4 * tex_x + 4 * TxW * tex_y;
            let bytes = &tex[tex_idx..(tex_idx + 4)];
            if z_buff[idx] < z {
                pixels[idx..idx + 4].clone_from_slice(bytes);
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
        let elapsed_2 = (sys_time.elapsed().unwrap().as_secs_f64() - elapsed).recip();
        println!("fps approx: {}", elapsed_2);
    }

    Ok(())
}
