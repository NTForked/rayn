use crate::math::{f32x4, Vec3, Wec3};
use std::iter::*;
use std::ops::*;

macro_rules! srgbs {
    ($($n:ident => $vt:ident, $tt:ident),+) => {
        $(#[derive(Clone, Copy, Debug)]
        pub struct $n(pub $vt);

        impl From<$vt> for $n {
            fn from(v: $vt) -> Self {
                $n(v)
            }
        }

        impl Deref for $n {
            type Target = $vt;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl $n {
            pub fn new(r: $tt, g: $tt, b: $tt) -> Self {
                $n($vt::new(r, g, b))
            }

            #[allow(dead_code)]
            pub fn gamma_corrected(&self, gamma: $tt) -> Self {
                $n(self.0.map(|x| x.powf($tt::from(1.0) / gamma)))
            }

            #[allow(dead_code)]
            pub fn saturated(&self) -> Self {
                $n(
                    self.0
                        .map(|x| x.min($tt::from(0.0)).min($tt::from(1.0))),
                )
            }

            pub fn zero() -> Self {
                $n($vt::zero())
            }

            pub fn one() -> Self {
                $n($vt::one())
            }

            pub fn max_channel(&self) -> $tt {
                self.0.component_max()
            }
        }

        impl Sum for $n {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold($n::zero(), |a, b| a + b)
            }
        })+
    };
}

srgbs!(WSrgb => Wec3, f32x4, Srgb => Vec3, f32);

impl Srgb {
    pub fn is_black(&self) -> bool {
        self.0.component_max() < 0.0001
    }

    pub fn is_nan(&self) -> bool {
        std::iter::once(self.x.classify())
            .chain(std::iter::once(self.y.classify()))
            .chain(std::iter::once(self.z.classify()))
            .fold(false, |acc, class| {
                acc || if let std::num::FpCategory::Nan = class {
                    true
                } else {
                    false
                }
            })
    }
}

impl WSrgb {
    pub fn merge(mask: f32x4, a: Self, b: Self) -> Self {
        Self(Wec3::merge(mask, a.0, b.0))
    }

    pub fn splat(srgb: Srgb) -> Self {
        Self(Wec3::splat(srgb.0))
    }
}

impl From<[Srgb; 4]> for WSrgb {
    fn from(srgbs: [Srgb; 4]) -> Self {
        WSrgb(Wec3::from([srgbs[0].0, srgbs[1].0, srgbs[2].0, srgbs[3].0]))
    }
}

impl Into<[Srgb; 4]> for WSrgb {
    fn into(self) -> [Srgb; 4] {
        let vecs: [Vec3; 4] = self.0.into();
        [Srgb(vecs[0]), Srgb(vecs[1]), Srgb(vecs[2]), Srgb(vecs[3])]
    }
}

macro_rules! impl_wrapper_ops {
    ($wrapper_t:ident => $tt:ident) => {
        impl ::std::ops::Add for $wrapper_t {
            type Output = $wrapper_t;

            fn add(self, other: $wrapper_t) -> $wrapper_t {
                $wrapper_t(self.0 + other.0)
            }
        }

        impl std::ops::AddAssign for $wrapper_t {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs
            }
        }

        impl ::std::ops::Sub for $wrapper_t {
            type Output = $wrapper_t;

            fn sub(self, other: $wrapper_t) -> $wrapper_t {
                $wrapper_t(self.0 - other.0)
            }
        }

        impl std::ops::SubAssign for $wrapper_t {
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs
            }
        }

        impl ::std::ops::Div<$tt> for $wrapper_t {
            type Output = $wrapper_t;

            fn div(self, other: $tt) -> $wrapper_t {
                $wrapper_t(self.0 / other)
            }
        }

        impl std::ops::DivAssign<$tt> for $wrapper_t {
            fn div_assign(&mut self, rhs: $tt) {
                *self = *self / rhs
            }
        }

        impl ::std::ops::Mul<$tt> for $wrapper_t {
            type Output = $wrapper_t;

            fn mul(self, other: $tt) -> $wrapper_t {
                $wrapper_t(self.0 * other)
            }
        }

        impl std::ops::MulAssign<$tt> for $wrapper_t {
            fn mul_assign(&mut self, rhs: $tt) {
                *self = *self * rhs
            }
        }

        impl ::std::ops::Mul<$wrapper_t> for $wrapper_t {
            type Output = $wrapper_t;

            fn mul(self, other: $wrapper_t) -> $wrapper_t {
                $wrapper_t(self.0 * other.0)
            }
        }

        impl std::ops::MulAssign for $wrapper_t {
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs
            }
        }
    };
}

impl_wrapper_ops!(Srgb => f32);
impl_wrapper_ops!(WSrgb => f32x4);
