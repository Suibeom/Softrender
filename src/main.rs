use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use rand::Rng;

use std::ops;

use std::time::SystemTime;
#[derive(Copy, Clone)]
struct Triangle<T, C>
where
    T: Copy + Clone,
    C: Copy + Clone,
{
    pt1: T,
    pt2: T,
    pt3: T,
    color: C,
}

#[derive(Copy, Clone)]
struct RGBA {
    c: u32,
}

impl RGBA {
    pub fn new(c: u32) -> RGBA {
        RGBA { c }
    }
}

#[derive(Copy, Clone)]
struct Vertex<T, U> {
    spatial: T,
    uv: U,
}

#[derive(Copy, Clone)]
struct Pt3 {
    x: f64,
    y: f64,
    z: f64,
}

struct ProjectionData {
    origin_pt: Pt3,
    plane_unit_normal: Pt3,
    plane_basis_x: Pt3,
    plane_basis_y: Pt3,
}
struct ViewportData {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    pixels_tall: usize,
    pixels_wide: usize,
}

impl ops::Add<Pt3> for Pt3 {
    type Output = Pt3;
    fn add(self, rhs: Pt3) -> Pt3 {
        return Pt3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        };
    }
}

impl ops::Sub<Pt3> for Pt3 {
    type Output = Pt3;
    fn sub(self, rhs: Pt3) -> Pt3 {
        return Pt3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        };
    }
}

impl ops::Mul<Pt3> for Pt3 {
    type Output = f64;
    fn mul(self, rhs: Pt3) -> f64 {
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
    }
}

impl ops::Mul<f64> for Pt3 {
    type Output = Pt3;
    fn mul(self, rhs: f64) -> Pt3 {
        return Pt3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        };
    }
}

impl ZAugmented for RasterPointDepth {
    fn get_z(&self) -> f64 {
        self.z
    }
}
#[derive(Copy, Clone)]
struct RasterPoint {
    x: usize,
    y: usize,
    clipped: bool,
}

#[derive(Copy, Clone)]
struct RasterPointDepth {
    x: usize,
    y: usize,
    clipped: bool,
    z: f64,
}

#[derive(Copy, Clone)]
struct Pt2 {
    x: f64,
    y: f64,
}

trait ZAugmented {
    fn get_z(&self) -> f64;
}

fn get_color() -> RGBA {
    let color = rand::thread_rng().gen::<u32>();
    let color = color & 0xFFFFFF;
    return RGBA { c: color };
}

fn make_triangle_partition(
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    grid_width: usize,
    grid_height: usize,
    jitter: f64,
) -> Vec<Triangle<Vertex<Pt3, ()>, RGBA>> {
    let float_pitch_x = (x_max - x_min) / grid_width as f64;
    let float_pitch_y = (y_max - y_min) / grid_height as f64;
    let mut points = vec![
        Pt3 {
            x: 0.0,
            y: 0.0,
            z: 0.0
        };
        (grid_height + 1) * (grid_width + 1)
    ];
    for i in 0..grid_height {
        for j in 0..grid_width {
            let (down, right): (bool, bool) = (rand::random(), rand::random());
            let x = match down {
                false => (i as f64 * float_pitch_x) + x_min,
                true => (i as f64 * float_pitch_x) + x_min + jitter,
            };
            let y = match right {
                false => (j as f64 * float_pitch_y) + y_min,
                true => (j as f64 * float_pitch_y) + y_min + jitter,
            };
            points[i + grid_height * j] = Pt3 { x, y, z: 1.0 };
        }
    }
    let mut triangles: Vec<Triangle<Vertex<Pt3, _>, RGBA>> = Vec::new();
    let ul_grid_height = (grid_height as i64 - 1) as usize;
    let ul_grid_width = (grid_width as i64 - 1) as usize;

    for i in 0..ul_grid_height {
        for j in 0..ul_grid_width {
            match rand::random() {
                // true => slice from i,j to i+1, j+1
                // false => slice from i, j+1 to i+1, j
                true => {
                    triangles.push(Triangle {
                        pt1: Vertex {
                            spatial: points[i + grid_height * j],
                            uv: (),
                        },
                        pt2: Vertex {
                            spatial: points[i + 1 + grid_height * j],
                            uv: (),
                        },
                        pt3: Vertex {
                            spatial: points[(i + 1) + grid_height * (j + 1)],
                            uv: (),
                        },
                        color: get_color(),
                    });
                    triangles.push(Triangle {
                        pt1: Vertex {
                            spatial: points[i + grid_height * j],
                            uv: (),
                        },
                        pt2: Vertex {
                            spatial: points[i + grid_height * (j + 1)],
                            uv: (),
                        },
                        pt3: Vertex {
                            spatial: points[i + 1 + grid_height * (j + 1)],
                            uv: (),
                        },
                        color: get_color(),
                    });
                }

                false => {
                    triangles.push(Triangle {
                        pt1: Vertex {
                            spatial: points[i + grid_height * j],
                            uv: (),
                        },
                        pt2: Vertex {
                            spatial: points[i + 1 + grid_height * j],
                            uv: (),
                        },
                        pt3: Vertex {
                            spatial: points[i + grid_height * (j + 1)],
                            uv: (),
                        },
                        color: get_color(),
                    });
                    triangles.push(Triangle {
                        pt1: Vertex {
                            spatial: points[(i + 1) + grid_height * j],
                            uv: (),
                        },
                        pt2: Vertex {
                            spatial: points[i + grid_height * (j + 1)],
                            uv: (),
                        },
                        pt3: Vertex {
                            spatial: points[i + 1 + grid_height * (j + 1)],
                            uv: (),
                        },
                        color: get_color(),
                    });
                }
            }
        }
    }

    return triangles;
}

fn simple_projection<T>(p: Vertex<Pt3, T>, proj: &ProjectionData) -> Vertex<Pt2, T> {
    let t = (1.0 - p.spatial * proj.plane_unit_normal)
        / ((p.spatial - proj.origin_pt) * proj.plane_unit_normal);
    let projected_point = proj.origin_pt * t + p.spatial * (1.0 - t);
    let plane_x = projected_point * proj.plane_basis_x;
    let plane_y = projected_point * proj.plane_basis_y;
    return Vertex {
        spatial: Pt2 {
            x: plane_x,
            y: plane_y,
        },
        uv: p.uv,
    };
}

fn simple_projection_with_z(
    p: Vertex<Pt3, RasterPoint>,
    proj: &ProjectionData,
) -> Vertex<Pt2, RasterPointDepth> {
    let t = (1.0 - p.spatial * proj.plane_unit_normal)
        / ((p.spatial - proj.origin_pt) * proj.plane_unit_normal);
    let z = p.spatial * proj.plane_unit_normal;
    let projected_point = proj.origin_pt * t + p.spatial * (1.0 - t);
    let plane_x = projected_point * proj.plane_basis_x;
    let plane_y = projected_point * proj.plane_basis_y;
    return Vertex {
        spatial: Pt2 {
            x: plane_x,
            y: plane_y,
        },
        uv: RasterPointDepth {
            x: p.uv.x,
            y: p.uv.y,
            clipped: p.uv.clipped,
            z,
        },
    };
}

fn spatial_to_pixel<T>(p: Vertex<Pt2, T>) -> Vertex<RasterPoint, T> {
    let v = ViewportData {
        x_min: -4.0,
        x_max: 4.0,
        y_min: -3.0,
        y_max: 3.0,
        pixels_tall: H,
        pixels_wide: W,
    };
    let plane_x = p.spatial.x;
    let plane_y = p.spatial.y;
    if (plane_x < v.x_min) | (plane_x > v.x_max) | (plane_y < v.y_min) | (plane_y > v.y_max) {
        return Vertex {
            spatial: RasterPoint {
                x: 0,
                y: 0,
                clipped: true,
            },
            uv: p.uv,
        };
    };
    let x = (v.pixels_wide as f64 * (plane_x - v.x_min) / (v.x_max - v.x_min)) as usize;
    let y = (v.pixels_tall as f64 * (v.y_max - plane_y) / (v.y_max - v.y_min)) as usize;
    return Vertex {
        spatial: RasterPoint {
            x,
            y,
            clipped: false,
        },
        uv: p.uv,
    };
}

fn prep_triangle<T, P>(
    t: Triangle<Vertex<Pt3, T>, RGBA>,
    proj: &P,
) -> Triangle<Vertex<RasterPoint, T>, RGBA>
where
    T: Copy,
    P: Fn(Vertex<Pt3, T>) -> Vertex<Pt2, T>,
{
    return normal_form(Triangle {
        pt1: spatial_to_pixel(proj(t.pt1)),
        pt2: spatial_to_pixel(proj(t.pt2)),
        pt3: spatial_to_pixel(proj(t.pt3)),
        color: t.color,
    });
}

fn draw_triangle<P, Q>(t: Triangle<Vertex<RasterPoint, Q>, RGBA>, plotter: &mut P)
where
    P: FnMut(usize, usize, RGBA) -> (),
    Q: Copy + Clone,
{
    let clipped = t.pt1.spatial.clipped | t.pt2.spatial.clipped | t.pt3.spatial.clipped;
    if clipped {
        return;
    };
    let y_start = t.pt1.spatial.y;
    let y_end = std::cmp::max(t.pt2.spatial.y, t.pt3.spatial.y);
    let y_mid = std::cmp::min(t.pt2.spatial.y, t.pt3.spatial.y);
    let (left_m, left_b) = slope_intercept(t.pt1.spatial, t.pt2.spatial);
    let (right_m, right_b) = slope_intercept(t.pt1.spatial, t.pt3.spatial);
    let left_bottom = t.pt2.spatial.y <= t.pt3.spatial.y;
    let (bottom_m, bottom_b) = slope_intercept(t.pt2.spatial, t.pt3.spatial);
    for y in y_start..y_end {
        let (x_left, x_right) = match y < y_mid {
            true => (
                (y as f64 * left_m + left_b) as usize,
                (y as f64 * right_m + right_b) as usize,
            ),
            false => match left_bottom {
                true => (
                    (y as f64 * bottom_m + bottom_b) as usize,
                    (y as f64 * right_m + right_b) as usize,
                ),

                false => (
                    (y as f64 * left_m + left_b) as usize,
                    (y as f64 * bottom_m + bottom_b) as usize,
                ),
            },
        };
        for x in x_left..x_right {
            plotter(x, y, t.color.clone());
        }
    }
}

fn slope_intercept(from: RasterPoint, to: RasterPoint) -> (f64, f64) {
    let slope = (from.x as f64 - to.x as f64) / (from.y as f64 - to.y as f64);
    let intercept = from.x as f64 - (from.y as f64 * slope);
    return (slope, intercept);
}

fn slope_intercept_perspective<T, U>(
    from: &Vertex<U, T>,
    to: &Vertex<U, T>,
    steps: u32,
) -> (f64, f64, T, T)
where
    T: ops::Add<T, Output = T> + ops::Mul<f64, Output = T> + ops::Sub<T, Output = T> + Copy,
    U: ZAugmented,
{
    let z1_inv = from.spatial.get_z().recip();
    let z2_inv = to.spatial.get_z().recip();
    let steps_inv = (steps as f64).recip();
    let z_inv_slope = (z2_inv - z1_inv) * steps_inv;
    let z_inv_intercept = z1_inv;
    let uv1_zinv = from.uv * z1_inv;
    let uv2_zinv = to.uv * z2_inv;
    let uv_zinv_slope = (uv2_zinv - uv1_zinv) * steps_inv;
    let uv_zinv_intercept = uv1_zinv;
    return (
        z_inv_slope,
        z_inv_intercept,
        uv_zinv_slope,
        uv_zinv_intercept,
    );
}

fn normal_form<T>(
    t: Triangle<Vertex<RasterPoint, T>, RGBA>,
) -> Triangle<Vertex<RasterPoint, T>, RGBA>
where
    T: Copy,
{
    let mut pts = [t.pt1, t.pt2, t.pt3];
    pts.sort_unstable_by(|a, b| {
        (a.spatial.x + W * a.spatial.y)
            .partial_cmp(&(b.spatial.x + W * b.spatial.y))
            .unwrap()
    });
    let pt1 = pts[0];
    let (v1_x, v1_y) = (
        pts[0].spatial.x as i64 - pts[1].spatial.x as i64,
        pts[0].spatial.y as i64 - pts[1].spatial.y as i64,
    );
    let (v2_x, v2_y) = (
        pts[0].spatial.x as i64 - pts[2].spatial.x as i64,
        pts[0].spatial.y as i64 - pts[2].spatial.y as i64,
    );
    let (pt2, pt3) = match v1_x * v2_y - v2_x * v1_y <= 0 {
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
    let sys_time = SystemTime::now();
    let tris = make_triangle_partition(-2.0, 2.0, -1.5, 2.5, 400, 400, 0.01);

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
            origin_pt: Pt3 {
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
                x: -y,
                y: x,
                z: 0.0,
            },
        };
        let mut pixels = [0u8; 4 * W * H];
        let mut plot = |x: usize, y: usize, color: RGBA| {
            let idx = 4 * x + 4 * W * y;
            let bytes = color.c.to_ne_bytes();
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
        let simple_proj = |t| simple_projection(t, &proj);
        for tri in &tris {
            draw_triangle(prep_triangle(*tri, &simple_proj), &mut plot);
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
