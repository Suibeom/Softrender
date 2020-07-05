#![feature(const_generics)]
use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;


struct Triangle<T, C> {
    pt1: T,
    pt2: T,
    pt3: T,
    color: C,
}

#[derive(Clone)]
struct RGBA {
    c: u32,
}

impl RGBA {
    pub fn new(c: u32) -> RGBA {
        RGBA { c }
    }
}

struct Pt3 {
    x: u32,
    y: u32,
    z: u32,
}

#[derive(Copy, Clone)]
struct Pt2 {
    x: usize,
    y: usize,
}

fn make_triangle_partition<
    const pixels_width: usize,
    const pixels_height: usize,
    const grid_width: usize,
    const grid_height: usize,
    const jitter: usize,
>() -> Result<[Triangle<Pt2, RGBA>; grid_height*grid_width*2 ], _> {
    if grid_height * grid_width == 0 {
        return Err
    };
    let float_pitch_x = pixels_width as f32 / grid_width as f32;
    let float_pitch_y = pixels_height as f32 / grid_height as f32;
    let mut points = [Pt2; grid_height * grid_width];
    for i in 1..grid_height {
        for j in 1..grid_width {
            points[i + grid_height * j] = Pt2 {
                x: i * float_pitch_x as usize,
                y: j * float_pitch_y as usize,
            };
        }
    }
    let mut triangles = [Triangle<Pt2, RGBA>; grid_height*grid_width*2];
}

fn draw_triangle<P>(t: Triangle<Pt2, RGBA>, plotter: &mut P)
where
    P: FnMut(Pt2, RGBA) -> (),
{
    let y_start = t.pt1.y;
    let y_end = std::cmp::max(t.pt2.y, t.pt3.y);
    let y_mid = std::cmp::min(t.pt2.y, t.pt3.y);
    let (left_m, left_b) = slope_intercept(t.pt1, t.pt2);
    let (right_m, right_b) = slope_intercept(t.pt1, t.pt3);
    let left_bottom = t.pt2.y <= t.pt3.y;
    let (bottom_m, bottom_b) = slope_intercept(t.pt2, t.pt3);
    for y in y_start..y_end {
        let (x_left, x_right) = match y < y_mid {
            true => (
                (y as f32 * left_m + left_b) as usize,
                (y as f32 * right_m + right_b) as usize,
            ),
            false => match left_bottom {
                true => (
                    (y as f32 * bottom_m + bottom_b) as usize,
                    (y as f32 * right_m + right_b) as usize,
                ),

                false => (
                    (y as f32 * left_m + left_b) as usize,
                    (y as f32 * bottom_m + bottom_b) as usize,
                ),
            },
        };
        for x in x_left..x_right {
            plotter(Pt2 { x, y }, t.color.clone());
        }
    }
    plotter(t.pt1, RGBA { c: 0xFFFFFF });
    plotter(t.pt2, RGBA { c: 0xFFFF00 });
    plotter(t.pt3, RGBA { c: 0x00FFFF });
}

fn slope_intercept(from: Pt2, to: Pt2) -> (f32, f32) {
    let slope = (from.x as f32 - to.x as f32) / (from.y as f32 - to.y as f32);
    let intercept = from.x as f32 - (from.y as f32 * slope);
    return (slope, intercept);
}

fn normal_form(t: Triangle<Pt2, RGBA>) -> Triangle<Pt2, RGBA> {
    let mut pts = [t.pt1, t.pt2, t.pt3];
    pts.sort_unstable_by(|a, b| (a.x + W * a.y).partial_cmp(&(b.x + W * b.y)).unwrap());
    let pt1 = pts[0];
    let (pt2, pt3) = match pts[1].x <= pts[2].x {
        true => (pts[1], pts[2]),
        false => (pts[2], pts[1]),
    };

    return Triangle {
        pt1,
        pt2,
        pt3,
        color: t.color,
    };
}

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
    let mut pixels = [0u8; 4 * W * H];
    let mut plot = |pt: Pt2, color: RGBA| {
        let idx = 4 * pt.x + 4 * W * pt.y;
        let bytes = color.c.to_ne_bytes();
        pixels[idx] = bytes[0];
        pixels[idx + 1] = bytes[1];
        pixels[idx + 2] = bytes[2];
        pixels[idx + 3] = bytes[3];
    };

    let t1 = Triangle {
        pt1: Pt2 { x: 5, y: 50 },
        pt2: Pt2 { x: 50, y: 75 },
        pt3: Pt2 { x: 10, y: 25 },
        color: RGBA::new(0xFF00FF),
    };
    let t1 = normal_form(t1);
    let t2 = Triangle {
        pt1: Pt2 { x: 100, y: 55 },
        pt2: Pt2 { x: 50, y: 75 },
        pt3: Pt2 { x: 110, y: 15 },
        color: RGBA::new(0xFFAAAA),
    };
    let t2 = normal_form(t2);
    draw_triangle(t1, &mut plot);
    draw_triangle(t2, &mut plot);
    texture
        .update(None, &pixels, 4 * W)
        .map_err(|e| e.to_string())?;

    // texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //     for y in 0..256 {
    //         for x in 0..256 {
    //             let offset = y * pitch + x * 3;
    //             buffer[offset] = x as u8;
    //             buffer[offset + 1] = y as u8;
    //             buffer[offset + 2] = y as u8;
    //         }
    //     }
    // })?;

    canvas.clear();
    canvas.copy(&texture, None, None)?;
    canvas.present();

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
    }

    Ok(())
}