use rand::Rng;

use std::cmp::Ordering;
use std::ops;

use crate::common::*;

#[derive(Copy, Clone)]
pub struct Triangle<T>
where
    T: Copy,
{
    pub points: [T; 3],
}

#[derive(Copy, Clone)]
pub struct Vertex<V>
where
    V: Copy,
{
    pub spatial: Pt3,
    pub variable: V,
}

#[derive(Copy, Clone)]
pub struct ProjectedVertex<V>
where
    V: Copy,
{
    spatial: Pt3,
    projected: Pt3,
    variable: V,
}
#[derive(Copy, Clone)]
pub struct RasterLocatedVertex<V> {
    spatial: Pt3,
    projected: Pt3,
    variable: V,
    raster_location: RasterPoint,
}

pub fn get_color() -> Pt3 {
    let color = rand::thread_rng().gen::<u32>();
    let bytes = color.to_ne_bytes();
    return Pt3 {
        x: bytes[1] as f64,
        y: bytes[2] as f64,
        z: bytes[3] as f64,
    };
}

fn simple_projection_with_z<T: Copy>(p: Vertex<T>, proj: &ProjectionData) -> ProjectedVertex<T> {
    let t = (1.0 - p.spatial * proj.plane_unit_normal)
        / ((p.spatial - proj.origin_pt) * proj.plane_unit_normal);
    let z = p.spatial * proj.plane_unit_normal;
    let projected_point = proj.origin_pt * t + p.spatial * (1.0 - t);
    let plane_x = projected_point * proj.plane_basis_x;
    let plane_y = projected_point * proj.plane_basis_y;
    return ProjectedVertex {
        projected: Pt3 {
            x: plane_x,
            y: plane_y,
            z,
        },
        spatial: p.spatial,
        variable: p.variable,
    };
}

const W: usize = 320;
const H: usize = 240;

pub const DEFAULT_VIEWPORT: ViewportData = ViewportData {
    x_min: -4.0,
    x_max: 4.0,
    y_min: -3.0,
    y_max: 3.0,
    pixels_tall: H,
    pixels_wide: W,
};

pub fn raster_locate_vertex<T: Copy>(
    p: ProjectedVertex<T>,
    v: &ViewportData,
) -> RasterLocatedVertex<T> {
    let plane_x = p.projected.x;
    let plane_y = p.projected.y;
    if (plane_x < v.x_min) | (plane_x > v.x_max) | (plane_y < v.y_min) | (plane_y > v.y_max) {
        return RasterLocatedVertex {
            raster_location: RasterPoint {
                x: 0,
                y: 0,
                clipped: true,
            },
            spatial: p.spatial,
            variable: p.variable,
            projected: p.projected,
        };
    };
    let x = (v.pixels_wide as f64 * (plane_x - v.x_min) / (v.x_max - v.x_min)) as usize;
    let y = (v.pixels_tall as f64 * (v.y_max - plane_y) / (v.y_max - v.y_min)) as usize;
    return RasterLocatedVertex {
        raster_location: RasterPoint {
            x,
            y,
            clipped: false,
        },
        spatial: p.spatial,
        variable: p.variable,
        projected: p.projected,
    };
}

fn slope_intercept<T>(from: T, to: T, steps: i32, start: i32) -> (T, T)
where
    T: ops::Add<T, Output = T> + ops::Mul<f64, Output = T> + ops::Sub<T, Output = T> + Copy,
{
    let steps_inv = (steps as f64).recip();
    let slope: T = (to - from) * steps_inv;
    let intercept = from - (slope * start as f64);
    return (slope, intercept);
}

fn slope_intercept_perspective<T>(
    from: T,
    to: T,
    from_zinv: f64,
    to_zinv: f64,
    steps: i32,
    start: i32,
) -> (T, T)
where
    T: ops::Add<T, Output = T> + ops::Mul<f64, Output = T> + ops::Sub<T, Output = T> + Copy,
{
    let uv1_zinv = from * from_zinv;
    let uv2_zinv = to * to_zinv;
    return slope_intercept(uv1_zinv, uv2_zinv, steps, start);
}

fn compare_raster_lex<T: Copy>(a: &RasterLocatedVertex<T>, b: &RasterLocatedVertex<T>) -> Ordering {
    (a.raster_location.x + W * a.raster_location.y)
        .partial_cmp(&(b.raster_location.x + W * b.raster_location.y))
        .unwrap()
}

pub fn prep_triangle<T: Copy>(
    t: Triangle<Vertex<T>>,
    viewport: &ViewportData,
    projection: &ProjectionData,
) -> Triangle<RasterLocatedVertex<T>> {
    return Triangle {
        points: [
            raster_locate_vertex::<T>(
                simple_projection_with_z::<T>(t.points[0], projection),
                viewport,
            ),
            raster_locate_vertex::<T>(
                simple_projection_with_z::<T>(t.points[1], projection),
                viewport,
            ),
            raster_locate_vertex::<T>(
                simple_projection_with_z::<T>(t.points[2], projection),
                viewport,
            ),
        ],
    };
}

pub fn draw_triangle<T, P>(t: Triangle<RasterLocatedVertex<T>>, plotter: &mut P)
where
    P: FnMut(usize, usize, f64, T) -> (),
    T: ops::Add<T, Output = T> + ops::Mul<f64, Output = T> + ops::Sub<T, Output = T> + Copy,
{
    let mut points: [RasterLocatedVertex<T>; 3] = t.points;
    let clipped = points[0].raster_location.clipped
        | points[1].raster_location.clipped
        | points[2].raster_location.clipped;
    if clipped {
        return;
    };
    points.sort_unstable_by(compare_raster_lex::<T>);

    let pt1: RasterLocatedVertex<T> = points[0];

    let (v1_x, v1_y) = (
        points[0].raster_location.x as i64 - points[1].raster_location.x as i64,
        points[0].raster_location.y as i64 - points[1].raster_location.y as i64,
    );
    let (v2_x, v2_y) = (
        points[0].raster_location.x as i64 - points[2].raster_location.x as i64,
        points[0].raster_location.y as i64 - points[2].raster_location.y as i64,
    );
    let (pt2, pt3) = match v1_x * v2_y - v2_x * v1_y >= 0 {
        true => (points[2], points[1]),
        false => (points[1], points[2]),
    };

    let y_start = pt1.raster_location.y;
    let y_end = std::cmp::max(pt2.raster_location.y, pt3.raster_location.y);
    let y_mid = std::cmp::min(pt2.raster_location.y, pt3.raster_location.y);
    let left_bottom = pt2.raster_location.y <= pt3.raster_location.y;
    let left_steps = pt2.raster_location.y as i32 - pt1.raster_location.y as i32;
    let right_steps = pt3.raster_location.y as i32 - pt1.raster_location.y as i32;
    let bottom_steps = pt3.raster_location.y as i32 - pt2.raster_location.y as i32;

    let (left_m, left_b) = slope_intercept(
        pt1.raster_location.x as f64,
        pt2.raster_location.x as f64,
        left_steps,
        pt1.raster_location.y as i32,
    );

    let (right_m, right_b) = slope_intercept(
        pt1.raster_location.x as f64,
        pt3.raster_location.x as f64,
        right_steps,
        pt1.raster_location.y as i32,
    );

    let (bottom_m, bottom_b) = slope_intercept(
        pt2.raster_location.x as f64,
        pt3.raster_location.x as f64,
        bottom_steps,
        pt2.raster_location.y as i32,
    );

    let (left_zinv_m, left_zinv_b) = slope_intercept(
        (pt1.projected.z as f64).recip(),
        (pt2.projected.z as f64).recip(),
        left_steps,
        pt1.raster_location.y as i32,
    );

    let (right_zinv_m, right_zinv_b) = slope_intercept(
        (pt1.projected.z as f64).recip(),
        (pt3.projected.z as f64).recip(),
        right_steps,
        pt1.raster_location.y as i32,
    );

    let (bottom_zinv_m, bottom_zinv_b) = slope_intercept(
        (pt2.projected.z as f64).recip(),
        (pt3.projected.z as f64).recip(),
        bottom_steps,
        pt2.raster_location.y as i32,
    );

    let (left_var_m, left_var_b) = slope_intercept_perspective(
        pt1.variable,
        pt2.variable,
        (pt1.projected.z as f64).recip(),
        (pt2.projected.z as f64).recip(),
        left_steps,
        pt1.raster_location.y as i32,
    );
    let (right_var_m, right_var_b) = slope_intercept_perspective(
        pt1.variable,
        pt3.variable,
        (pt1.projected.z as f64).recip(),
        (pt3.projected.z as f64).recip(),
        right_steps,
        pt1.raster_location.y as i32,
    );
    let (bottom_var_m, bottom_var_b) = slope_intercept_perspective(
        pt2.variable,
        pt3.variable,
        (pt2.projected.z as f64).recip(),
        (pt3.projected.z as f64).recip(),
        bottom_steps,
        pt2.raster_location.y as i32,
    );

    for y in y_start..y_end {
        let (x_left, x_right, var_left, var_right, zinv_left, zinv_right) = match y < y_mid {
            true => (
                (left_m * (y as i64) as f64 + left_b) as usize,
                (right_m * (y as i64) as f64 + right_b) as usize,
                (left_var_m * (y as f64) + left_var_b),
                (right_var_m * (y as f64) + right_var_b),
                (left_zinv_m * (y as f64) + left_zinv_b),
                (right_zinv_m * (y as f64) + right_zinv_b),
            ),
            false => match left_bottom {
                true => (
                    (bottom_m * (y as i64) as f64 + bottom_b) as usize,
                    (right_m * (y as i64) as f64 + right_b) as usize,
                    (bottom_var_m * (y as f64) + bottom_var_b),
                    (right_var_m * (y as f64) + right_var_b),
                    (bottom_zinv_m * (y as f64) + bottom_zinv_b),
                    (right_zinv_m * (y as f64) + right_zinv_b),
                ),
                false => (
                    (left_m * (y as i64) as f64 + left_b) as usize,
                    (bottom_m * (y as i64) as f64 + bottom_b) as usize,
                    (left_var_m * (y as f64) + left_var_b),
                    (bottom_var_m * (y as f64) + bottom_var_b),
                    (left_zinv_m * (y as f64) + left_zinv_b),
                    (bottom_zinv_m * (y as f64) + bottom_zinv_b),
                ),
            },
        };
        let x_steps = x_right as i32 - x_left as i32;
        let (scanline_var_slope, scanline_var_intercept) =
            slope_intercept(var_left, var_right, x_steps, x_left as i32);
        let (scanline_zinv_slope, scanline_zinv_intercept) =
            slope_intercept(zinv_left, zinv_right, x_steps, x_left as i32);
        for x in x_left..x_right {
            let var = scanline_var_slope * x as f64 + scanline_var_intercept;
            let zinv = scanline_zinv_slope * x as f64 + scanline_zinv_intercept;
            let z = zinv.recip();
            plotter(x, y, z, var * z);
        }
    }
}

pub fn make_triangle_partition(
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    grid_width: usize,
    grid_height: usize,
    jitter: f64,
) -> Vec<Triangle<Vertex<Pt3>>> {
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
            let (down, right, out): (bool, bool, bool) =
                (rand::random(), rand::random(), rand::random());
            let x = match down {
                false => (i as f64 * float_pitch_x) + x_min,
                true => (i as f64 * float_pitch_x) + x_min + jitter,
            };
            let y = match right {
                false => (j as f64 * float_pitch_y) + y_min,
                true => (j as f64 * float_pitch_y) + y_min + jitter,
            };
            let z = match out {
                false => 1.0,
                true => 1.0 + jitter,
            };
            points[i + grid_height * j] = Pt3 { x, y, z };
        }
    }
    let mut triangles: Vec<Triangle<Vertex<Pt3>>> = Vec::new();
    let ul_grid_height = (grid_height as i64 - 1) as usize;
    let ul_grid_width = (grid_width as i64 - 1) as usize;

    for i in 0..ul_grid_height {
        for j in 0..ul_grid_width {
            match rand::random() {
                // true => slice from i,j to i+1, j+1
                // false => slice from i, j+1 to i+1, j
                true => {
                    triangles.push(Triangle {
                        points: [
                            Vertex {
                                spatial: points[i + grid_height * j],
                                variable: points[i + grid_height * j], // get_color(),
                            },
                            Vertex {
                                spatial: points[i + 1 + grid_height * j],
                                variable: points[i + 1 + grid_height * j], //get_color(),
                            },
                            Vertex {
                                spatial: points[i + 1 + grid_height * (j + 1)],
                                variable: points[i + 1 + grid_height * (j + 1)], //get_color(),
                            },
                        ],
                    });
                    triangles.push(Triangle {
                        points: [
                            Vertex {
                                spatial: points[i + grid_height * j],
                                variable: points[i + grid_height * j], //get_color(),
                            },
                            Vertex {
                                spatial: points[i + grid_height * (j + 1)],
                                variable: points[i + grid_height * (j + 1)], // get_color(),
                            },
                            Vertex {
                                spatial: points[i + 1 + grid_height * (j + 1)],
                                variable: points[i + 1 + grid_height * (j + 1)], //get_color(),
                            },
                        ],
                    });
                }

                false => {
                    triangles.push(Triangle {
                        points: [
                            Vertex {
                                spatial: points[i + grid_height * j],
                                variable: points[i + grid_height * j], //get_color(),
                            },
                            Vertex {
                                spatial: points[i + 1 + grid_height * j],
                                variable: points[i + 1 + grid_height * j], //get_color(),
                            },
                            Vertex {
                                spatial: points[i + grid_height * (j + 1)],
                                variable: points[i + grid_height * (j + 1)], //get_color(),
                            },
                        ],
                    });
                    triangles.push(Triangle {
                        points: [
                            Vertex {
                                spatial: points[i + 1 + grid_height * j],
                                variable: points[i + 1 + grid_height * j], //get_color(),
                            },
                            Vertex {
                                spatial: points[i + grid_height * (j + 1)],
                                variable: points[i + grid_height * (j + 1)], //get_color(),
                            },
                            Vertex {
                                spatial: points[i + 1 + grid_height * (j + 1)],
                                variable: points[i + 1 + grid_height * (j + 1)], //get_color(),
                            },
                        ],
                    });
                }
            }
        }
    }

    return triangles;
}

#[cfg(test)]
mod tests {
    #[test]
    fn slopes_work() {
        let (m, b) = crate::varvar::slope_intercept(0.0, 10.0, 10, 0);
        assert_eq!(m, 1.0);
        assert_eq!(b, 0.0);
        let (m, b) = crate::varvar::slope_intercept(10.0, 0.0, 10, 0);
        assert_eq!(m, -1.0);
        assert_eq!(b, 10.0);
    }
}
