use crate::chain::{Bounds, Chain, FixedLink, Parameters, RotatingLink, SlidingLink, Transform, Vec3};
use crate::gui::{draw_aabb, draw_transform};

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::path::Path;
use enum_map::{Enum, EnumMap};
use na::{Point3, Translation3, UnitQuaternion};
use ncollide3d::bounding_volume::bounding_volume::BoundingVolume;
//use ncollide3d::shape::{Compound, Cuboid, ShapeHandle};

#[derive(Enum, Copy, Clone)]
pub enum Parameter {
    X,
    Y,
    Z,
    Spindle,
}

impl Bounds<Parameter> for Parameter {
    fn bounded(&self, new_value: f32) -> f32 {
        match self {
            Parameter::X => new_value.max(0.0).min(1.0),
            Parameter::Y => new_value.max(0.0).min(0.5),
            Parameter::Z => new_value.max(-0.1).min(0.1),
            Parameter::Spindle => (new_value + std::f32::consts::PI * 2.0) % (std::f32::consts::PI * 2.0),
        }
    }
}

pub struct MPCNC {
    frame: SceneNode,
    x_tube: SceneNode,
    y_tube: SceneNode,
    z_axis: SceneNode,
    spindle: SceneNode,
    endmill: SceneNode,
    chain: Chain<Parameter>,
}

impl MPCNC {
    pub fn new(window: &mut Window, resources_dir: &Path) -> MPCNC {
        let mm = na::Vector3::new(0.001, 0.001, 0.001);

        let base_link = FixedLink::new(&Transform::translation(0.00, 0.00, 0.00), None);
        let x_link = SlidingLink::new(&Vec3::x_axis(), Parameter::X);
        let y_link = SlidingLink::new(&Vec3::y_axis(), Parameter::Y);
        let z_link = SlidingLink::new(&Vec3::z_axis(), Parameter::Z);

        let z_axis_link = FixedLink::new(
            &Transform::from_parts(
                Translation3::new(0.0, 0.0, 0.0),
                UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 0.0_f32.to_radians()) * 
                UnitQuaternion::from_axis_angle(&Vec3::y_axis(), 0.0_f32.to_radians()),
            ),
            None,
        );

        let spindle_link = FixedLink::new(
            &Transform::from_parts(
                Translation3::new(0.09, -0.09, 0.0),
                UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 0.0_f32.to_radians()) * 
                UnitQuaternion::from_axis_angle(&Vec3::y_axis(), 0.0_f32.to_radians()),
            ),
            None,
        );

        let rotation_link = RotatingLink::new(&Vec3::z_axis(), 0.0_f32.to_radians(), Parameter::Spindle);

        let endmill_link = FixedLink::new(
            &Transform::from_parts(
                Translation3::new(0.0, 0.0, 0.0),
                UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 0.0_f32.to_radians()) * 
                UnitQuaternion::from_axis_angle(&Vec3::y_axis(), 0.0_f32.to_radians()),
            ),
            None,
        );

        let endmill_tip_link = FixedLink::new(
            &Transform::from_parts(
                Translation3::new(0.0, 0.0, -0.035),
                UnitQuaternion::identity(),
            ),
            None,
        );

        let chain = Chain::new(
            vec![
                Box::new(base_link),
                Box::new(x_link),
                Box::new(y_link),
                Box::new(z_axis_link),
                Box::new(z_link),
                Box::new(spindle_link),
                Box::new(rotation_link),
                Box::new(endmill_link),
                Box::new(endmill_tip_link),
            ],
            vec![], //(0, 4), (6, 7), (9, 10), (12, 13)],
        );

        let mut mpcnc = MPCNC {
            frame: window.add_obj(&resources_dir.join("base.obj"), resources_dir, mm),
            x_tube: window.add_obj(&resources_dir.join("gantry-x-tube.obj"), resources_dir, mm),
            y_tube: window.add_obj(&resources_dir.join("gantry-y-tube.obj"), resources_dir, mm),
            z_axis: window.add_obj(&resources_dir.join("z-axis.obj"), resources_dir, mm),
            spindle: window.add_obj(&resources_dir.join("spindle.obj"), resources_dir, mm),
            endmill: window.add_obj(&resources_dir.join("endmill.obj"), resources_dir, mm),
            chain: chain,
        };

        mpcnc.frame.set_color(1.0, 1.0, 0.0);
        mpcnc.x_tube.set_color(1.0, 1.0, 0.0);
        mpcnc.y_tube.set_color(1.0, 1.0, 0.0);
        mpcnc.z_axis.set_color(0.0, 0.0, 1.0);
        mpcnc.spindle.set_color(0.0, 1.0, 0.0);
        mpcnc.endmill.set_color(1.0, 1.0, 1.0);

        mpcnc.set_visible(false);
        
        mpcnc
    }

    #[allow(dead_code)]
    pub fn get_chain(&self) -> &Chain<Parameter> {
        &self.chain
    }

    pub fn get_endmill_tip(&self, parameters: &Parameters<Parameter>) -> Vec3 {
        let end_poses = self.chain.compute_all_end_poses(parameters);
        end_poses[8].translation.vector
    }


    pub fn get_default_parameters(&self) -> Parameters<Parameter> {
        let mut params = EnumMap::new();

        params[Parameter::X] = 0.5;
        params[Parameter::Y] = 0.25;
        params
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.frame.set_visible(visible);
        self.x_tube.set_visible(visible);
        self.y_tube.set_visible(visible);
        self.z_axis.set_visible(visible);
        self.spindle.set_visible(visible);
        self.endmill.set_visible(visible);
    }

    pub fn render(&mut self, window: &mut Window, parameters: &Parameters<Parameter>, show_transforms: bool, show_collision_bbox: bool) {
        if show_transforms {
            let end_poses = self.chain.compute_all_end_poses(parameters);

            for pose in end_poses.iter() {
                draw_transform(window, pose, 0.1);
            }
        }

        let start_poses = self.chain.compute_all_start_poses(parameters);
        let end_poses = self.chain.compute_all_end_poses(parameters);

        self.set_visible(true);

        if show_collision_bbox {
            let links = self.chain.get_links();

            for (link, pose) in links.iter().zip(start_poses.iter()) {
                if let Some(shape) = link.get_collision_shape() {
                    let aabb = shape.aabb(pose);
                    draw_aabb(window, &aabb.tightened(0.04), &Point3::new(1.0, 1.0, 1.0));
                }
            }
        }

        self.frame.set_local_transformation(end_poses[0]);
        self.x_tube.set_local_transformation(end_poses[1]);
        self.y_tube.set_local_transformation(end_poses[2] * Transform::translation(-end_poses[2].translation.x, 0.0, 0.0));
        self.z_axis.set_local_transformation(end_poses[4]);
        self.spindle.set_local_transformation(end_poses[5]);
        self.endmill.set_local_transformation(end_poses[7]);
    }
}
