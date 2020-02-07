use crate::spectrum::WSrgb;

use std::f32::consts::PI;
pub use ultraviolet::f32x4;

use vek::vec;

pub type Vec3 = ultraviolet::Vec3;
pub type Wec3 = ultraviolet::Wec3;
pub type Vec2 = ultraviolet::Vec2;
pub type Wec2 = ultraviolet::Wec2;
pub type Vec2u = vec::repr_c::Vec2<usize>;
pub type Aabru = vek::geom::repr_c::Aabr<usize>;
pub type Extent2u = vek::vec::repr_c::Extent2<usize>;

pub type Wat3 = ultraviolet::Wat3;

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Wec3,
    // pub orientation: Quat,
}

pub trait OrthonormalBasis<M>: Sized {
    fn get_orthonormal_basis(&self) -> M;
}

impl OrthonormalBasis<Wat3> for Wec3 {
    fn get_orthonormal_basis(&self) -> Wat3 {
        let nor = *self;
        let ks = nor.z.signum();
        let ka = f32x4::ONE / (f32x4::ONE + nor.z.abs());
        let kb = -ks * nor.x * nor.y * ka;
        let uu = Wec3::new(f32x4::ONE - nor.x * nor.x * ka, ks * kb, -ks * nor.x);
        let vv = Wec3::new(kb, ks - nor.y * nor.y * ka * ks, -nor.y);
        Wat3::new(uu, vv, nor)
    }
}

pub trait RandomSample2d {
    type Sample;
    fn rand_in_unit_disk(samples: &Self::Sample) -> Self;
}

impl RandomSample2d for Wec2 {
    type Sample = [f32x4; 2];
    fn rand_in_unit_disk(samples: &Self::Sample) -> Self {
        let rho = samples[0].sqrt();
        let theta = samples[1] * f32x4::from(2f32 * PI);
        let (sin, cos) = theta.sin_cos();
        Wec2::new(rho * cos, rho * sin)
    }
}

pub trait RandomSample3d<T> {
    type Sample;
    fn rand_in_unit_sphere(samples: &Self::Sample) -> Self;
    fn rand_on_unit_sphere(samples: &Self::Sample) -> Self;
    fn cosine_weighted_in_hemisphere(samples: &Self::Sample, factor: T) -> Self;
}

impl RandomSample3d<f32x4> for Wec3 {
    type Sample = [f32x4; 2];
    fn rand_in_unit_sphere(samples: &Self::Sample) -> Self {
        let theta = samples[0] * f32x4::from(2f32 * PI);
        let phi = samples[1] * f32x4::from(2.0) - f32x4::ONE;
        let ophisq = (f32x4::ONE - phi * phi).sqrt();
        let (sin, cos) = theta.sin_cos();
        Wec3::new(ophisq * cos, ophisq * sin, phi)
    }

    fn rand_on_unit_sphere(samples: &Self::Sample) -> Self {
        Self::rand_in_unit_sphere(samples).normalized()
    }

    fn cosine_weighted_in_hemisphere(samples: &Self::Sample, constriction: f32x4) -> Self {
        let xy = Wec2::rand_in_unit_disk(samples) * constriction;
        let z = (f32x4::ONE - xy.mag_sq()).sqrt();
        Wec3::new(xy.x, xy.y, z)
    }
}

#[allow(dead_code)]
pub fn f0_from_ior(ior: f32x4) -> f32x4 {
    let f0 = (f32x4::ONE - ior) / (f32x4::ONE + ior);
    f0 * f0
}

pub fn f_schlick(cos: f32x4, f0: f32x4) -> f32x4 {
    f0 + (f32x4::ONE - f0) * (f32x4::ONE - cos).powi([5, 5, 5, 5])
}

pub fn f_schlick_c(cos: f32x4, f0: WSrgb) -> WSrgb {
    f0 + (WSrgb::one() - f0) * (f32x4::ONE - cos).powi([5, 5, 5, 5])
}

#[allow(dead_code)]
pub fn saturate(v: f32x4) -> f32x4 {
    v.min(f32x4::ONE).max(f32x4::ZERO)
}

pub struct CDF {
    items: Vec<(f32, f32)>,
    densities: Vec<f32>,
    weight_sum: f32,
    prepared: bool,
}

impl CDF {
    pub fn new() -> Self {
        CDF {
            items: Vec::new(),
            densities: Vec::new(),
            weight_sum: 0.0,
            prepared: false,
        }
    }

    pub fn insert(&mut self, item: f32, weight: f32) {
        self.items.push((item, weight));
        self.weight_sum += weight;
    }

    pub fn prepare(&mut self) {
        if self.prepared {
            return;
        }

        for (_, weight) in self.items.iter_mut() {
            *weight /= self.weight_sum;
        }

        let mut cum = 0.0;
        for (_, weight) in self.items.iter() {
            cum += *weight;
            self.densities.push(cum);
        }

        for (&(_, weight), density) in self.items.iter().zip(self.densities.iter_mut()).rev() {
            *density = 1.0;
            if weight > 0.0 {
                break;
            }
        }

        self.prepared = true;
    }

    pub fn sample(&self, x: f32) -> Option<(f32, f32)> {
        for (ret, density) in self.items.iter().zip(self.densities.iter()) {
            if *density >= x {
                return Some(*ret);
            }
        }
        None
    }
}
