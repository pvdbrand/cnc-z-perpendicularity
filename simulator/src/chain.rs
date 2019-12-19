use std::vec::Vec;

use enum_map::{Enum, EnumMap};
use na::geometry::UnitQuaternion;
use na::{Dynamic, Isometry3, MatrixMN, Unit, Vector3, VectorN, U7};
use ncollide3d::shape::Shape;
use std::collections::HashSet;
use std::iter::FromIterator;

pub type Transform = Isometry3<f32>;
pub type Vec3 = Vector3<f32>;

pub type Parameters<P> = EnumMap<P, f32>;

pub trait Bounds<P> {
    fn bounded(&self, new_value: f32) -> f32;
}

pub trait Link<P: Enum<f32>> {
    fn get_local_transform(&self, parameters: &Parameters<P>) -> Transform;
    fn get_collision_shape(&self) -> &Option<Box<dyn Shape<f32>>>;
}

pub struct Chain<P> {
    links: Vec<Box<dyn Link<P>>>,
    ignore_collision_link_indices: HashSet<(usize, usize)>,
}

impl<P> Chain<P>
where
    P: Enum<f32> + Copy + Bounds<P>,
    <P as Enum<f32>>::Array: Clone,
{
    pub fn new<T>(links: Vec<Box<dyn Link<P>>>, ignore_collision_link_indices: T) -> Self
    where
        T: IntoIterator<Item = (usize, usize)>,
    {
        Chain {
            links,
            ignore_collision_link_indices: HashSet::from_iter(ignore_collision_link_indices),
        }
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

    pub fn get_links(&self) -> &Vec<Box<dyn Link<P>>> {
        &self.links
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

    #[allow(dead_code)]
    pub fn compute_self_collisions(&self, parameters: &Parameters<P>) -> Vec<(usize, usize)> {
        let start_poses = self.compute_all_start_poses(parameters);
        let mut result = Vec::new();

        for i in 0..self.links.len() {
            if let Some(shape_i) = self.links[i].get_collision_shape() {
                let pose_i = &start_poses[i];

                for j in i + 1..self.links.len() {
                    if !self.ignore_collision_link_indices.contains(&(i, j)) && !self.ignore_collision_link_indices.contains(&(j, i)) {
                        if let Some(shape_j) = self.links[j].get_collision_shape() {
                            let pose_j = &start_poses[j];
                            let d = ncollide3d::query::distance(pose_i, &**shape_i, pose_j, &**shape_j);

                            if d == 0.0 {
                                result.push((i, j));
                            }
                        }
                    }
                }
            }
        }

        result
    }

    #[allow(dead_code)]
    pub fn forward_kinematics(&self, parameters: &Parameters<P>) -> Transform {
        let mut result = Transform::identity();

        for link in self.links.iter() {
            result *= link.get_local_transform(parameters);
        }

        result
    }

    #[allow(dead_code)]
    pub fn inverse_kinematics(
        &self,
        parameters: &Parameters<P>,
        target: &Transform,
        max_iterations: usize,
        distance_threshold: f32,
    ) -> Option<Parameters<P>> {
        if self.links.is_empty() {
            return None;
        }

        let mut current = parameters.clone();

        for iteration in 0..=max_iterations {
            let t = self.forward_kinematics(&current);
            let e = transform_difference(&t, &target);

            if e.norm() <= distance_threshold {
                return Some(current);
            }

            if iteration < max_iterations {
                self.inverse_jacobian_step(&mut current, &e);
            }
        }

        None
    }

    fn compute_jacobian(&self, parameters: &mut Parameters<P>) -> MatrixMN<f32, U7, Dynamic> {
        let mut result = MatrixMN::<f32, U7, Dynamic>::zeros(parameters.len());
        let current_end_pose = self.forward_kinematics(parameters);
        let epsilon = 0.001;

        for c in 0..P::POSSIBLE_VALUES {
            let param = P::from_usize(c);

            let old = parameters[param];
            parameters[param] += epsilon;
            let new_end_pose = self.forward_kinematics(parameters);
            parameters[param] = old;

            let diff = transform_difference(&current_end_pose, &new_end_pose) / epsilon;
            result.set_column(c, &diff);
        }

        result
    }

    fn inverse_jacobian_step(&self, parameters: &mut Parameters<P>, e: &TransformDifference) {
        let jacobian = self.compute_jacobian(parameters);
        let inverse_jacobian = jacobian
            .svd(true, true)
            .pseudo_inverse(1e-4)
            .expect("Could not compute pseudo inverse of Jacobian");

        debug_assert!(inverse_jacobian.nrows() == parameters.len());
        debug_assert!(inverse_jacobian.ncols() == 7);

        for (r, (param, value)) in parameters.iter_mut().enumerate() {
            *value = param.bounded(*value + inverse_jacobian.row(r).transpose().dot(e));
        }
    }
}

type TransformDifference = VectorN<f32, U7>;

fn transform_difference(a: &Transform, b: &Transform) -> TransformDifference {
    let mut result = VectorN::<f32, U7>::zeros();
    let dr = b.rotation.coords - a.rotation.coords;
    let dt = b.translation.vector - a.translation.vector;

    debug_assert!(dr.len() == 4);
    debug_assert!(dt.len() == 3);

    for (r, v) in dr.iter().enumerate() {
        result[r] = *v;
    }
    for (r, v) in dt.iter().enumerate() {
        result[r + 4] = *v;
    }

    result
}

// Fixed link -----------------------------------------------------------------

pub struct FixedLink {
    transform: Transform,
    collision_shape: Option<Box<dyn Shape<f32>>>,
}

impl FixedLink {
    pub fn new(transform: &Transform, collision_shape: Option<Box<dyn Shape<f32>>>) -> Self {
        FixedLink {
            transform: *transform,
            collision_shape: collision_shape,
        }
    }
}

impl<P: Enum<f32>> Link<P> for FixedLink {
    fn get_local_transform(&self, _parameters: &Parameters<P>) -> Transform {
        self.transform
    }

    fn get_collision_shape(&self) -> &Option<Box<dyn Shape<f32>>> {
        &self.collision_shape
    }
}

// Sliding link ---------------------------------------------------------------

pub struct SlidingLink<P: Enum<f32> + Copy> {
    axis: Unit<Vec3>,
    parameter: P,
}

impl<P: Enum<f32> + Copy> SlidingLink<P> {
    pub fn new(axis: &Unit<Vec3>, parameter: P) -> Self {
        SlidingLink { axis: *axis, parameter }
    }
}

impl<P: Enum<f32> + Copy> Link<P> for SlidingLink<P> {
    fn get_local_transform(&self, parameters: &Parameters<P>) -> Transform {
        let param = parameters[self.parameter];

        Transform::translation(self.axis[0] * param, self.axis[1] * param, self.axis[2] * param)
    }

    fn get_collision_shape(&self) -> &Option<Box<dyn Shape<f32>>> {
        &None
    }
}

// Rotating link --------------------------------------------------------------

pub struct RotatingLink<P: Enum<f32> + Copy> {
    axis: Unit<Vec3>,
    center_angle: f32,
    parameter: P,
}

impl<P: Enum<f32> + Copy> RotatingLink<P> {
    pub fn new(axis: &Unit<Vec3>, center_angle: f32, parameter: P) -> Self {
        RotatingLink {
            axis: *axis,
            center_angle,
            parameter,
        }
    }
}

impl<P: Enum<f32> + Copy> Link<P> for RotatingLink<P> {
    fn get_local_transform(&self, parameters: &Parameters<P>) -> Transform {
        Transform::identity() * UnitQuaternion::from_axis_angle(&self.axis, parameters[self.parameter] + self.center_angle)
    }

    fn get_collision_shape(&self) -> &Option<Box<dyn Shape<f32>>> {
        &None
    }
}
