use std::ops;

#[derive(Copy, Clone)]
pub struct Pt3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Copy, Clone)]
pub struct Pt2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Copy, Clone)]
pub struct RasterPoint {
    pub x: usize,
    pub y: usize,
    pub clipped: bool,
}

pub struct ProjectionData {
    pub origin_pt: Pt3,
    pub plane_unit_normal: Pt3,
    pub plane_basis_x: Pt3,
    pub plane_basis_y: Pt3,
}
pub struct ViewportData {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub pixels_tall: usize,
    pub pixels_wide: usize,
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
