use std::cmp::Ordering;
use std::ops;

use crate::common::*;

#[derive(Copy, Clone)]
pub struct Triangle<T> {
    points: [T; 3],
}

#[derive(Copy, Clone)]
pub struct Vertex<V> {
    spatial: Pt3,
    variable: V,
}

#[derive(Copy, Clone)]
pub struct ProjectedVertex<V> {
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

fn simple_projection_with_z<T>(p: Vertex<T>, proj: &ProjectionData) -> ProjectedVertex<T> {
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

const default_viewport: ViewportData = ViewportData {
    x_min: -4.0,
    x_max: 4.0,
    y_min: -3.0,
    y_max: 3.0,
    pixels_tall: H,
    pixels_wide: W,
};

pub fn raster_locate_vertex<T>(p: ProjectedVertex<T>, v: ViewportData) -> RasterLocatedVertex<T> {
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

fn slope_intercept<T>(from: T, to: T, steps: u32) -> (T, T)
where
    T: ops::Add<T, Output = T> + ops::Mul<f64, Output = T> + ops::Sub<T, Output = T> + Copy,
{
    let steps_inv = (steps as f64).recip();
    let slope = (from - to) * steps_inv;
    let intercept = from;
    return (slope, intercept);
}

fn slope_intercept_perspective<T>(
    from: &RasterLocatedVertex<T>,
    to: &RasterLocatedVertex<T>,
    steps: u32,
) -> (T, T)
where
    T: ops::Add<T, Output = T> + ops::Mul<f64, Output = T> + ops::Sub<T, Output = T> + Copy,
{
    let z1_inv = from.projected.z.recip();
    let z2_inv = to.projected.z.recip();
    let uv1_zinv = from.variable * z1_inv;
    let uv2_zinv = to.variable * z2_inv;
    return slope_intercept(uv1_zinv, uv2_zinv, steps);
}

fn compare_raster_lex<T>(a: &RasterLocatedVertex<T>, b: &RasterLocatedVertex<T>) -> Ordering {
    (a.raster_location.x + W * a.raster_location.y)
        .partial_cmp(&(b.raster_location.x + W * b.raster_location.y))
        .unwrap()
}

pub fn draw_triangle<T, P>(t: Triangle<RasterLocatedVertex<T>>, plotter: &mut P)
where
    P: FnMut(usize, usize, T) -> (),
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
    let (pt2, pt3) = match v1_x * v2_y - v2_x * v1_y <= 0 {
        true => (points[1], points[2]),
        false => (points[2], points[1]),
    };

    let y_start = pt1.raster_location.y;
    let y_end = std::cmp::max(pt2.raster_location.y, pt3.raster_location.y);
    let y_mid = std::cmp::min(pt2.raster_location.y, pt3.raster_location.y);
    let left_bottom = pt2.raster_location.y <= pt3.raster_location.y;
    let (y_left, y_right) = match left_bottom {
        true => (y_end, y_mid),
        false => (y_mid, y_end),
    };
    let left_steps = (y_left as i64 - y_start as i64) as u32;
    let right_steps = (y_right as i64 - y_start as i64) as u32;
    let bottom_steps = (y_end as i64 - y_mid as i64) as u32;

    let (left_m, left_b) = slope_intercept(
        pt1.raster_location.y as f64,
        pt2.raster_location.y as f64,
        left_steps,
    );
    let (right_m, right_b) = slope_intercept(
        pt1.raster_location.y as f64,
        pt3.raster_location.y as f64,
        right_steps,
    );
    let (bottom_m, bottom_b) = slope_intercept(
        pt2.raster_location.y as f64,
        pt3.raster_location.y as f64,
        bottom_steps,
    );

    let (left_var_m, left_var_b) = slope_intercept(pt1.variable, pt2.variable, left_steps);
    let (right_var_m, right_var_b) = slope_intercept(pt1.variable, pt3.variable, right_steps);
    let (bottom_var_m, bottom_var_b) = slope_intercept(pt2.variable, pt3.variable, bottom_steps);

    for y in y_start..y_end {
        let (x_left, x_right, var_left, var_right) = match y < y_mid {
            true => (
                (left_m * y as f64 + left_b) as usize,
                (right_m * y as f64 + right_b) as usize,
                (left_var_m * y as f64 + left_var_b),
                (right_var_m * y as f64 + right_var_b),
            ),
            false => match left_bottom {
                true => (
                    (bottom_m * y as f64 + bottom_b) as usize,
                    (right_m * y as f64 + right_b) as usize,
                    (bottom_var_m * y as f64 + bottom_var_b),
                    (right_var_m * y as f64 + right_var_b),
                ),

                false => (
                    (left_m * y as f64 + left_b) as usize,
                    (bottom_m * y as f64 + bottom_b) as usize,
                    (left_var_m * y as f64 + left_var_b),
                    (bottom_var_m * y as f64 + bottom_var_b),
                ),
            },
        };
        let x_steps = (x_right as i64 - x_left as i64) as u32;
        let (scanline_var_slope, scanline_var_intercept) =
            slope_intercept(var_left, var_right, x_steps);
        for x in x_left..x_right {
            let var = scanline_var_slope * x as f64 + scanline_var_intercept;
            plotter(x, y, var);
        }
    }
}
