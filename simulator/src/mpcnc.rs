use crate::chain::{Bounds, Chain, FixedLink, Parameters, RotatingLink, SlidingLink, Transform, Vec3};
use crate::gui::{draw_transform};

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::path::Path;
use enum_map::{Enum, EnumMap};
use ncollide3d::shape::{Shape, Compound, ShapeHandle, Capsule};
use na::{Translation3, UnitQuaternion};

#[derive(Enum, Copy, Clone)]
pub enum Parameter {
    X,
    Y,
    Z,
    Spindle,

    ZAxisX,
    ZAxisY,
    SpindleX,
    SpindleY,
    EndmillX,
    EndmillY,
    EndmillOffset,
}

impl Bounds<Parameter> for Parameter {
    fn bounded(&self, new_value: f32) -> f32 {
        match self {
            Parameter::X => new_value.max(0.0).min(1.0),
            Parameter::Y => new_value.max(0.0).min(0.5),
            Parameter::Z => new_value.max(-0.045).min(0.0),
            Parameter::EndmillOffset => new_value.max(0.0).min(0.01),
            _ => (new_value + std::f32::consts::PI * 2.0) % (std::f32::consts::PI * 2.0),
        }
    }
}

pub struct MPCNC {
    frame: SceneNode,
    spoilboard: SceneNode,
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

        let base_link = FixedLink::new(&Transform::translation(0.0, 0.0, 0.14));
        let x_link = SlidingLink::new(&Vec3::x_axis(), Parameter::X);
        let y_link = SlidingLink::new(&Vec3::y_axis(), Parameter::Y);
        let z_link = SlidingLink::new(&Vec3::z_axis(), Parameter::Z);

        let z_axis_offset_link = FixedLink::new(&Transform::translation(-0.09, 0.09, 0.0));
        let z_axis_x_link = RotatingLink::new(&Vec3::x_axis(), 0.0_f32.to_radians(), Parameter::ZAxisX);
        let z_axis_y_link = RotatingLink::new(&Vec3::y_axis(), 0.0_f32.to_radians(), Parameter::ZAxisY);
        let z_axis_offset_inv_link = FixedLink::new(&Transform::translation(0.09, -0.09, -0.05));

        let spindle_offset_link = FixedLink::new(&Transform::translation(0.0, 0.0, -0.015 + 0.185 / 2.0));
        let spindle_x_link = RotatingLink::new(&Vec3::x_axis(), 0.0_f32.to_radians(), Parameter::SpindleX);
        let spindle_y_link = RotatingLink::new(&Vec3::y_axis(), 0.0_f32.to_radians(), Parameter::SpindleY);
        let spindle_rotation_link = RotatingLink::new(&Vec3::z_axis(), 0.0_f32.to_radians(), Parameter::Spindle);
        let spindle_offset_inv_link = FixedLink::new(&Transform::translation(0.0, 0.0, -0.185 / 2.0));

        let endmill_offset_link = SlidingLink::new(&Vec3::x_axis(), Parameter::EndmillOffset);
        let endmill_x_link = RotatingLink::new(&Vec3::x_axis(), 0.0_f32.to_radians(), Parameter::EndmillX);
        let endmill_y_link = RotatingLink::new(&Vec3::y_axis(), 0.0_f32.to_radians(), Parameter::EndmillY);
        let endmill_tip_link = FixedLink::new(&Transform::translation(0.0, 0.0, -0.03));
        let end_effector_link = FixedLink::new(&Transform::translation(0.0, 0.0, 0.0));

        let chain = Chain::new(vec![
            Box::new(base_link),
            Box::new(x_link),
            Box::new(y_link),
            Box::new(z_axis_offset_link),
            Box::new(z_axis_x_link),
            Box::new(z_axis_y_link),
            Box::new(z_link),
            Box::new(z_axis_offset_inv_link),
            Box::new(spindle_offset_link),
            Box::new(spindle_x_link),
            Box::new(spindle_y_link),
            Box::new(spindle_rotation_link),
            Box::new(spindle_offset_inv_link),
            Box::new(endmill_offset_link),
            Box::new(endmill_x_link),
            Box::new(endmill_y_link),
            Box::new(endmill_tip_link),
            Box::new(end_effector_link),
        ]);

        let mut mpcnc = MPCNC {
            frame: window.add_obj(&resources_dir.join("frame.obj"), resources_dir, mm),
            spoilboard: window.add_obj(&resources_dir.join("spoilboard.obj"), resources_dir, mm),
            x_tube: window.add_obj(&resources_dir.join("gantry-x-tube.obj"), resources_dir, mm),
            y_tube: window.add_obj(&resources_dir.join("gantry-y-tube.obj"), resources_dir, mm),
            z_axis: window.add_obj(&resources_dir.join("z-axis.obj"), resources_dir, mm),
            spindle: window.add_obj(&resources_dir.join("spindle.obj"), resources_dir, mm),
            endmill: window.add_obj(&resources_dir.join("endmill.obj"), resources_dir, mm),
            chain: chain,
        };

        mpcnc.frame.set_color(1.0, 1.0, 0.0);
        mpcnc.spoilboard.set_color(0.25, 0.25, 0.25);
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

    pub fn get_end_effector_pos(&self, parameters: &Parameters<Parameter>) -> Transform {
        let end_poses = self.chain.compute_all_end_poses(parameters);
        end_poses[end_poses.len() - 1]
    }

    pub fn get_probe_collision_shape(&self, parameters: &Parameters<Parameter>) -> Box<dyn Shape<f32>> {
        let end_poses = self.chain.compute_all_end_poses(parameters);
        let pose = end_poses[end_poses.len() - 1];
        let collision_pose = Transform::from_parts(
            Translation3::new(0.0, 0.0, 0.03 / 2.0), 
            UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 90.0_f32.to_radians())
        );

        Box::new(Compound::new(vec![(
            pose * collision_pose,
            ShapeHandle::new(Capsule::new(0.03 / 2.0 - 0.004 / 2.0, 0.004 / 2.0)),
        )]))
    }

    pub fn get_default_parameters(&self) -> Parameters<Parameter> {
        let mut params = EnumMap::new();

        params[Parameter::X] = 0.50;
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

    pub fn render(&mut self, window: &mut Window, parameters: &Parameters<Parameter>, show_transforms: bool) {
        if show_transforms {
            let end_poses = self.chain.compute_all_end_poses(parameters);

            for pose in end_poses.iter() {
                draw_transform(window, pose, 0.1);
            }
        }

        let start_poses = self.chain.compute_all_start_poses(parameters);

        self.set_visible(true);

        // if show_collision_bbox {
        //     let links = self.chain.get_links();

        //     for (link, pose) in links.iter().zip(start_poses.iter()) {
        //         if let Some(shape) = link.get_collision_shape() {
        //             let aabb = shape.aabb(pose);
        //             draw_aabb(window, &aabb.tightened(0.0), &Point3::new(1.0, 1.0, 1.0));
        //         }
        //     }
        // }

        self.frame.set_local_transformation(start_poses[0]);
        self.spoilboard.set_local_transformation(start_poses[0]);
        self.x_tube.set_local_transformation(start_poses[2]);
        self.y_tube.set_local_transformation(start_poses[3] * Transform::translation(-start_poses[3].translation.x, 0.0, 0.0));
        self.z_axis.set_local_transformation(start_poses[8]);
        self.spindle.set_local_transformation(start_poses[13]);
        self.endmill.set_local_transformation(start_poses[17]);
    }
}
