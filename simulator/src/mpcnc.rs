use crate::chain::{Bounds, Chain, FixedLink, Parameters, RotatingLink, SlidingLink, Transform, Vec3};
use crate::gui::{draw_transform};
use crate::probe::Probe;

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::path::Path;
use enum_map::{Enum, EnumMap};
use na::{Translation3, UnitQuaternion};
use ncollide3d::shape::ShapeHandle;

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
    fn bounded(&self, new_value: f64) -> f64 {
        match self {
            Parameter::X => new_value.max(0.0).min(1.0),
            Parameter::Y => new_value.max(0.0).min(0.5),
            Parameter::Z => new_value.max(-0.045).min(0.0),
            Parameter::EndmillOffset => new_value.max(0.0).min(0.150),
            _ => (new_value + std::f64::consts::PI * 2.0) % (std::f64::consts::PI * 2.0),
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
    arm: SceneNode,
    endmill: SceneNode,
    endmill_tip: SceneNode,
    endmill_collision_shape: ShapeHandle<f64>,
    endmill_tip_collision_shape: ShapeHandle<f64>,
    endmill_index: usize,
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
        let z_axis_x_link = RotatingLink::new(&Vec3::x_axis(), 0.0_f64.to_radians(), Parameter::ZAxisX);
        let z_axis_y_link = RotatingLink::new(&Vec3::y_axis(), 0.0_f64.to_radians(), Parameter::ZAxisY);
        let z_axis_offset_inv_link = FixedLink::new(&Transform::translation(0.09, -0.09, -0.05));

        let spindle_offset_link = FixedLink::new(&Transform::translation(0.0, 0.0, -0.01 + 0.185 / 2.0));
        let spindle_x_link = RotatingLink::new(&Vec3::x_axis(), 0.0_f64.to_radians(), Parameter::SpindleX);
        let spindle_y_link = RotatingLink::new(&Vec3::y_axis(), 0.0_f64.to_radians(), Parameter::SpindleY);
        let spindle_rotation_link = RotatingLink::new(&Vec3::z_axis(), 0.0_f64.to_radians(), Parameter::Spindle);
        let spindle_offset_inv_link = FixedLink::new(&Transform::translation(0.0, 0.0, -0.185 / 2.0));

        let endmill_offset_link = SlidingLink::new(&Vec3::x_axis(), Parameter::EndmillOffset);
        let endmill_x_link = RotatingLink::new(&Vec3::x_axis(), 0.0_f64.to_radians(), Parameter::EndmillX);
        let endmill_y_link = RotatingLink::new(&Vec3::y_axis(), 0.0_f64.to_radians(), Parameter::EndmillY);
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

        let tip_diam   = 0.0002;
        let tip_length = 0.0005;
        let (endmill_trimesh, endmill_collision_shape) = Probe::get_cylinder_shape(0.004, 0.030 - tip_length, &Transform::from_parts(
            Translation3::new(0.0, 0.0, (0.030 - tip_length) / 2.0 + tip_length), 
            UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 90.0_f64.to_radians())
        ));
        let (endmill_tip_trimesh, endmill_tip_collision_shape) = Probe::get_cylinder_shape(tip_diam, tip_length, &Transform::from_parts(
            Translation3::new(0.0, 0.0, tip_length / 2.0), 
            UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 90.0_f64.to_radians())
        ));

        let mut mpcnc = MPCNC {
            frame: window.add_obj(&resources_dir.join("frame.obj"), resources_dir, mm),
            spoilboard: window.add_obj(&resources_dir.join("spoilboard.obj"), resources_dir, mm),
            x_tube: window.add_obj(&resources_dir.join("gantry-x-tube.obj"), resources_dir, mm),
            y_tube: window.add_obj(&resources_dir.join("gantry-y-tube.obj"), resources_dir, mm),
            z_axis: window.add_obj(&resources_dir.join("z-axis.obj"), resources_dir, mm),
            spindle: window.add_obj(&resources_dir.join("spindle.obj"), resources_dir, mm),
            arm: window.add_obj(&resources_dir.join("arm.obj"), resources_dir, mm),
            endmill: window.add_trimesh(endmill_trimesh, na::Vector3::from_element(1.0_f32)),
            endmill_tip: window.add_trimesh(endmill_tip_trimesh, na::Vector3::from_element(1.0_f32)),
            endmill_collision_shape: endmill_collision_shape,
            endmill_tip_collision_shape: endmill_tip_collision_shape,
            endmill_index: 17,
            chain: chain,
        };

        mpcnc.frame.set_color(0.5, 0.5, 0.5);
        mpcnc.spoilboard.set_color(0.25, 0.25, 0.25);
        mpcnc.x_tube.set_color(0.5, 0.5, 0.5);
        mpcnc.y_tube.set_color(0.5, 0.5, 0.5);
        mpcnc.z_axis.set_color(0.0, 0.0, 1.0);
        mpcnc.spindle.set_color(0.0, 1.0, 0.0);
        mpcnc.arm.set_color(0.0, 1.0, 0.0);
        mpcnc.endmill.set_color(1.0, 0.0, 0.0);
        mpcnc.endmill_tip.set_color(1.0, 0.0, 0.0);

        mpcnc
    }

    #[allow(dead_code)]
    pub fn get_chain(&self) -> &Chain<Parameter> {
        &self.chain
    }

    pub fn get_end_effector_pos(&self, parameters: &Parameters<Parameter>) -> Transform {
        self.chain.compute_all_start_poses(parameters)[self.endmill_index]
    }

    pub fn get_probe(&self, parameters: &Parameters<Parameter>) -> Probe {
        Probe::new(vec![
            (self.get_end_effector_pos(parameters), self.endmill_collision_shape.clone()),
            (self.get_end_effector_pos(parameters), self.endmill_tip_collision_shape.clone())
        ])
    }

    pub fn get_default_parameters(&self) -> Parameters<Parameter> {
        let mut params = EnumMap::new();

        params[Parameter::X] = 0.50;
        params[Parameter::Y] = 0.25;
        params
    }

    pub fn render(&mut self, window: &mut Window, parameters: &Parameters<Parameter>, show_transforms: bool) {
        if show_transforms {
            let end_poses = self.chain.compute_all_end_poses(parameters);

            for pose in end_poses.iter() {
                draw_transform(window, pose, 0.1);
            }
        }

        let start_poses = self.chain.compute_all_start_poses(parameters);

        self.frame.set_local_transformation(na::convert(start_poses[0]));
        self.spoilboard.set_local_transformation(na::convert(start_poses[0]));
        self.x_tube.set_local_transformation(na::convert(start_poses[2]));
        self.y_tube.set_local_transformation(na::convert(start_poses[3] * Transform::translation(-start_poses[3].translation.x, 0.0, 0.0)));
        self.z_axis.set_local_transformation(na::convert(start_poses[8]));
        self.spindle.set_local_transformation(na::convert(start_poses[13]));
        self.arm.set_local_transformation(na::convert(start_poses[13]));
        
        self.endmill.set_local_transformation(na::convert(self.get_end_effector_pos(parameters)));
        self.endmill_tip.set_local_transformation(na::convert(self.get_end_effector_pos(parameters)));
    }
}
