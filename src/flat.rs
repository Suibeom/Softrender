use crate::common::*;
use rand::Rng;

pub fn get_color() -> RGBA {
    let color = rand::thread_rng().gen::<u32>();
    let color = color & 0xFFFFFF;
    return RGBA { c: color };
}

#[derive(Copy, Clone)]
pub struct Triangle<T, C>
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
pub struct RGBA {
    pub c: u32,
}

impl RGBA {
    pub fn new(c: u32) -> RGBA {
        RGBA { c }
    }
}

const W: usize = 320;
const H: usize = 240;

pub fn make_triangle_partition(
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    grid_width: usize,
    grid_height: usize,
    jitter: f64,
) -> Vec<Triangle<Pt3, RGBA>> {
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
    let mut triangles: Vec<Triangle<Pt3, RGBA>> = Vec::new();
    let ul_grid_height = (grid_height as i64 - 1) as usize;
    let ul_grid_width = (grid_width as i64 - 1) as usize;

    for i in 0..ul_grid_height {
        for j in 0..ul_grid_width {
            match rand::random() {
                // true => slice from i,j to i+1, j+1
                // false => slice from i, j+1 to i+1, j
                true => {
                    triangles.push(Triangle {
                        pt1: points[i + grid_height * j],

                        pt2: points[i + 1 + grid_height * j],

                        pt3: points[(i + 1) + grid_height * (j + 1)],

                        color: get_color(),
                    });
                    triangles.push(Triangle {
                        pt1: points[i + grid_height * j],

                        pt2: points[i + grid_height * (j + 1)],

                        pt3: points[i + 1 + grid_height * (j + 1)],

                        color: get_color(),
                    });
                }

                false => {
                    triangles.push(Triangle {
                        pt1: points[i + grid_height * j],

                        pt2: points[i + 1 + grid_height * j],

                        pt3: points[i + grid_height * (j + 1)],

                        color: get_color(),
                    });
                    triangles.push(Triangle {
                        pt1: points[(i + 1) + grid_height * j],

                        pt2: points[i + grid_height * (j + 1)],

                        pt3: points[i + 1 + grid_height * (j + 1)],

                        color: get_color(),
                    });
                }
            }
        }
    }

    return triangles;
}

pub fn normal_form(t: Triangle<RasterPoint, RGBA>) -> Triangle<RasterPoint, RGBA> {
    let mut pts = [t.pt1, t.pt2, t.pt3];
    pts.sort_unstable_by(|a, b| (a.x + W * a.y).partial_cmp(&(b.x + W * b.y)).unwrap());
    let pt1 = pts[0];
    let (v1_x, v1_y) = (
        pts[0].x as i64 - pts[1].x as i64,
        pts[0].y as i64 - pts[1].y as i64,
    );
    let (v2_x, v2_y) = (
        pts[0].x as i64 - pts[2].x as i64,
        pts[0].y as i64 - pts[2].y as i64,
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

pub fn spatial_to_pixel(p: Pt2) -> RasterPoint {
    let v = ViewportData {
        x_min: -4.0,
        x_max: 4.0,
        y_min: -3.0,
        y_max: 3.0,
        pixels_tall: H,
        pixels_wide: W,
    };
    let plane_x = p.x;
    let plane_y = p.y;
    if (plane_x < v.x_min) | (plane_x > v.x_max) | (plane_y < v.y_min) | (plane_y > v.y_max) {
        return RasterPoint {
            x: 0,
            y: 0,
            clipped: true,
        };
    };
    let x = (v.pixels_wide as f64 * (plane_x - v.x_min) / (v.x_max - v.x_min)) as usize;
    let y = (v.pixels_tall as f64 * (v.y_max - plane_y) / (v.y_max - v.y_min)) as usize;
    return RasterPoint {
        x,
        y,
        clipped: false,
    };
}

pub fn simple_projection(p: Pt3, proj: &ProjectionData) -> Pt2 {
    let t = (1.0 - p * proj.plane_unit_normal) / ((p - proj.origin_pt) * proj.plane_unit_normal);
    let projected_point = proj.origin_pt * t + p * (1.0 - t);
    let plane_x = projected_point * proj.plane_basis_x;
    let plane_y = projected_point * proj.plane_basis_y;
    return Pt2 {
        x: plane_x,
        y: plane_y,
    };
}

pub fn prep_triangle<P>(t: Triangle<Pt3, RGBA>, proj: &P) -> Triangle<RasterPoint, RGBA>
where
    P: Fn(Pt3) -> Pt2,
{
    return normal_form(Triangle {
        pt1: spatial_to_pixel(proj(t.pt1)),
        pt2: spatial_to_pixel(proj(t.pt2)),
        pt3: spatial_to_pixel(proj(t.pt3)),
        color: t.color,
    });
}

pub fn draw_triangle<P>(t: Triangle<RasterPoint, RGBA>, plotter: &mut P)
where
    P: FnMut(usize, usize, RGBA) -> (),
{
    let clipped = t.pt1.clipped | t.pt2.clipped | t.pt3.clipped;
    if clipped {
        return;
    };
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

pub fn slope_intercept(from: RasterPoint, to: RasterPoint) -> (f64, f64) {
    let slope = (from.x as f64 - to.x as f64) / (from.y as f64 - to.y as f64);
    let intercept = from.x as f64 - (from.y as f64 * slope);
    return (slope, intercept);
}
