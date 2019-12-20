use std::vec::Vec;

use enum_map::{Enum, EnumMap};
use na::geometry::UnitQuaternion;
use na::{Isometry3, Unit, Vector3};

pub type Transform = Isometry3<f64>;
pub type Vec3 = Vector3<f64>;

pub type Parameters<P> = EnumMap<P, f64>;

pub trait Bounds<P> {
    fn bounded(&self, new_value: f64) -> f64;
}

pub trait Link<P: Enum<f64>> {
    fn get_local_transform(&self, parameters: &Parameters<P>) -> Transform;
}

pub struct Chain<P> {
    links: Vec<Box<dyn Link<P>>>,
}

impl<P> Chain<P>
where
    P: Enum<f64> + Copy + Bounds<P>,
    <P as Enum<f64>>::Array: Clone,
{
    pub fn new(links: Vec<Box<dyn Link<P>>>) -> Self {
        Chain { links }
    }

    pub fn compute_all_end_poses(&self, parameters: &Parameters<P>) -> Vec<Transform> {
        let n = self.links.len();
        let mut t = Transform::identity();
        let mut result = Vec::with_capacity(n);

        for link in self.links.iter() {
            t *= link.get_local_transform(parameters);
            result.push(t);
        }

        result
    }

    pub fn compute_all_start_poses(&self, parameters: &Parameters<P>) -> Vec<Transform> {
        let n = self.links.len();
        let mut t = Transform::identity();
        let mut result = Vec::with_capacity(n);

        for link in self.links.iter() {
            result.push(t);
            t *= link.get_local_transform(parameters);
        }

        result
    }
}

// Fixed link -----------------------------------------------------------------

pub struct FixedLink {
    transform: Transform,
}

impl FixedLink {
    pub fn new(transform: &Transform) -> Self {
        FixedLink {
            transform: *transform,
        }
    }
}

impl<P: Enum<f64>> Link<P> for FixedLink {
    fn get_local_transform(&self, _parameters: &Parameters<P>) -> Transform {
        self.transform
    }
}

// Sliding link ---------------------------------------------------------------

pub struct SlidingLink<P: Enum<f64> + Copy> {
    axis: Unit<Vec3>,
    parameter: P,
}

impl<P: Enum<f64> + Copy> SlidingLink<P> {
    pub fn new(axis: &Unit<Vec3>, parameter: P) -> Self {
        SlidingLink { axis: *axis, parameter }
    }
}

impl<P: Enum<f64> + Copy> Link<P> for SlidingLink<P> {
    fn get_local_transform(&self, parameters: &Parameters<P>) -> Transform {
        let param = parameters[self.parameter];

        Transform::translation(self.axis[0] * param, self.axis[1] * param, self.axis[2] * param)
    }
}

// Rotating link --------------------------------------------------------------

pub struct RotatingLink<P: Enum<f64> + Copy> {
    axis: Unit<Vec3>,
    center_angle: f64,
    parameter: P,
}

impl<P: Enum<f64> + Copy> RotatingLink<P> {
    pub fn new(axis: &Unit<Vec3>, center_angle: f64, parameter: P) -> Self {
        RotatingLink {
            axis: *axis,
            center_angle,
            parameter,
        }
    }
}

impl<P: Enum<f64> + Copy> Link<P> for RotatingLink<P> {
    fn get_local_transform(&self, parameters: &Parameters<P>) -> Transform {
        Transform::identity() * UnitQuaternion::from_axis_angle(&self.axis, parameters[self.parameter] + self.center_angle)
    }
}
