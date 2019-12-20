use crate::chain::{Transform, Vec3};
use crate::probe::Probe;

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::path::Path;
use ncollide3d::shape::ShapeHandle;
use na::{Translation3, UnitQuaternion};

pub struct CalibrationObject {
    pos: Transform,

    plastic: SceneNode,
    wire_x: SceneNode,
    wire_y: SceneNode,
    bolt_c: SceneNode,
    bolt_x: SceneNode,
    bolt_y: SceneNode,

    wire_x_shape: ShapeHandle<f32>,
    wire_y_shape: ShapeHandle<f32>,
    bolt_c_shape: ShapeHandle<f32>,
    bolt_x_shape: ShapeHandle<f32>,
    bolt_y_shape: ShapeHandle<f32>,
}

impl CalibrationObject {
    pub fn new(window: &mut Window, resources_dir: &Path) -> CalibrationObject {
        let mm = na::Vector3::new(0.001, 0.001, 0.001);
        let diameter = 0.0005;
        let length = 0.040;

        let (wire_x_mesh, wire_x_shape) = Probe::get_cylinder_shape(diameter, length, &Transform::from_parts(
            Translation3::new(-0.020 + length / 2.0, -0.0115, 0.036),
            UnitQuaternion::from_axis_angle(&Vec3::z_axis(), 270.0_f32.to_radians())
        ));

        let (wire_y_mesh, wire_y_shape) = Probe::get_cylinder_shape(diameter, length, &Transform::from_parts(
            Translation3::new(-0.0215, -0.010 + length / 2.0, 0.036), 
            UnitQuaternion::identity()
        ));

        let (bolt_c_mesh, bolt_c_shape) = Probe::get_cylinder_shape(0.006, 0.0075, &Transform::from_parts(
            Translation3::new(-0.020, -0.010, 0.035), 
            UnitQuaternion::identity()
        ));

        let (bolt_x_mesh, bolt_x_shape) = Probe::get_cylinder_shape(0.006, 0.0075, &Transform::from_parts(
            Translation3::new(0.020, -0.010, 0.035), 
            UnitQuaternion::identity()
        ));

        let (bolt_y_mesh, bolt_y_shape) = Probe::get_cylinder_shape(0.006, 0.0075, &Transform::from_parts(
            Translation3::new(-0.020, 0.030, 0.035), 
            UnitQuaternion::identity()
        ));

        let mut object = CalibrationObject {
            pos: Transform::translation(0.52, 0.26, 0.0),
            plastic: window.add_obj(&resources_dir.join("calibration-object-plastic.obj"), resources_dir, mm),
            wire_x: window.add_trimesh(wire_x_mesh, Vec3::from_element(1.0)),
            wire_y: window.add_trimesh(wire_y_mesh, Vec3::from_element(1.0)),
            bolt_c: window.add_trimesh(bolt_c_mesh, Vec3::from_element(1.0)),
            bolt_x: window.add_trimesh(bolt_x_mesh, Vec3::from_element(1.0)),
            bolt_y: window.add_trimesh(bolt_y_mesh, Vec3::from_element(1.0)),
            wire_x_shape,
            wire_y_shape,
            bolt_c_shape,
            bolt_x_shape,
            bolt_y_shape,
        };

        object.plastic.set_color(1.0, 1.0, 0.0);
        object.wire_x.set_color(1.0, 0.0, 0.0);
        object.wire_y.set_color(1.0, 0.0, 0.0);
        object.bolt_c.set_color(1.0, 0.0, 0.0);
        object.bolt_x.set_color(1.0, 0.0, 0.0);
        object.bolt_y.set_color(1.0, 0.0, 0.0);
        
        object
    }

    pub fn get_probe(&self) -> Probe {
        Probe::new(vec![
            (self.pos, self.wire_x_shape.clone()),
            (self.pos, self.wire_y_shape.clone()),
            (self.pos, self.bolt_c_shape.clone()),
            (self.pos, self.bolt_x_shape.clone()),
            (self.pos, self.bolt_y_shape.clone()),
        ])
    }

    pub fn render(&mut self) {
        self.plastic.set_local_transformation(self.pos);
        self.wire_x.set_local_transformation(self.pos);
        self.wire_y.set_local_transformation(self.pos);
        self.bolt_c.set_local_transformation(self.pos);
        self.bolt_x.set_local_transformation(self.pos);
        self.bolt_y.set_local_transformation(self.pos);
    }
}
