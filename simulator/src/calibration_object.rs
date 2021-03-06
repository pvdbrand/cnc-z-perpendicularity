use crate::chain::{Transform, Vec3};
use crate::probe::Probe;

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::path::Path;
use ncollide3d::shape::ShapeHandle;
use na::{Translation3, UnitQuaternion};

pub trait CalibrationObject {
    fn get_probe(&self) -> Probe;
    fn render(&mut self);
}

pub struct FeelerGauge {
    pos: Transform,

    gauge_holder: SceneNode,
    gauge: SceneNode,

    gauge_shape: ShapeHandle<f64>,
}

pub struct TwoWires {
    pos: Transform,

    plastic: SceneNode,
    wire_x: SceneNode,
    wire_y: SceneNode,
    bolt_c: SceneNode,
    bolt_x: SceneNode,
    bolt_y: SceneNode,

    wire_x_shape: ShapeHandle<f64>,
    wire_y_shape: ShapeHandle<f64>,
    bolt_c_shape: ShapeHandle<f64>,
    bolt_x_shape: ShapeHandle<f64>,
    bolt_y_shape: ShapeHandle<f64>,
}

impl FeelerGauge {
    #[allow(dead_code)]
    pub fn new(window: &mut Window, resources_dir: &Path) -> Box<dyn CalibrationObject> {
        let mm = na::Vector3::new(0.001, 0.001, 0.001);
        let x = 89.0 / 1000.0;
        let y = 13.0 / 1000.0;
        let z =  0.8 / 1000.0;

        let (mesh, shape) = Probe::get_box_shape(x, y, z, &Transform::from_parts(
            Translation3::new(0.0, 0.0, 0.025),
            //UnitQuaternion::from_axis_angle(&Vec3::z_axis(), 2.0_f64.to_radians()) * UnitQuaternion::from_axis_angle(&Vec3::y_axis(), 1.0_f64.to_radians())  * UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 1.5_f64.to_radians())
            UnitQuaternion::identity()
        ));

        let mut object = FeelerGauge {
            pos: Transform::translation(0.50, 0.25, 0.0),
            gauge_holder: window.add_obj(&resources_dir.join("gauge-holder.obj"), resources_dir, mm),
            gauge: window.add_trimesh(mesh, na::Vector3::from_element(1.0_f32)),
            gauge_shape: shape,
        };

        object.gauge_holder.set_color(1.0, 1.0, 0.0);
        object.gauge.set_color(1.0, 0.0, 0.0);
        
        Box::new(object)
    }
}

impl CalibrationObject for FeelerGauge {
    fn get_probe(&self) -> Probe {
        Probe::new(vec![
            (self.pos, self.gauge_shape.clone()),
        ])
    }

    fn render(&mut self) {
        self.gauge_holder.set_local_transformation(na::convert(self.pos));
        self.gauge.set_local_transformation(na::convert(self.pos));
    }
}

impl TwoWires {
    #[allow(dead_code)]
    pub fn new(window: &mut Window, resources_dir: &Path) -> Box<dyn CalibrationObject> {
        let mm = na::Vector3::new(0.001, 0.001, 0.001);
        let diameter = 0.1 / 1000.0;
        let length = 0.040;

        let (wire_x_mesh, wire_x_shape) = Probe::get_cylinder_shape(diameter, length, &Transform::from_parts(
            Translation3::new(-0.020 + length / 2.0, -0.0115, 0.036),
            UnitQuaternion::from_axis_angle(&Vec3::z_axis(), 270.0_f64.to_radians())
        ));

        let (wire_y_mesh, wire_y_shape) = Probe::get_cylinder_shape(diameter, length, &Transform::from_parts(
            Translation3::new(-0.0215, -0.010 + length / 2.0, 0.036), 
            //UnitQuaternion::from_axis_angle(&Vec3::z_axis(), 3.0_f64.atan2(40.0)) * UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 2.0_f64.atan2(40.0))
            UnitQuaternion::identity()
        ));

        let (bolt_c_mesh, bolt_c_shape) = Probe::get_cylinder_shape(0.006, 0.0075, &Transform::from_parts(
            Translation3::new(-0.020, -0.010, 0.035 + 0.0075 / 2.0), 
            UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 90.0_f64.to_radians())
        ));

        let (bolt_x_mesh, bolt_x_shape) = Probe::get_cylinder_shape(0.006, 0.0075, &Transform::from_parts(
            Translation3::new(0.020, -0.010, 0.035 + 0.0075 / 2.0), 
            UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 90.0_f64.to_radians())
        ));

        let (bolt_y_mesh, bolt_y_shape) = Probe::get_cylinder_shape(0.006, 0.0075, &Transform::from_parts(
            Translation3::new(-0.020, 0.030, 0.035 + 0.0075 / 2.0), 
            UnitQuaternion::from_axis_angle(&Vec3::x_axis(), 90.0_f64.to_radians())
        ));

        let mut object = TwoWires {
            pos: Transform::translation(0.52, 0.26, 0.0),
            plastic: window.add_obj(&resources_dir.join("calibration-object-plastic.obj"), resources_dir, mm),
            wire_x: window.add_trimesh(wire_x_mesh, na::Vector3::from_element(1.0_f32)),
            wire_y: window.add_trimesh(wire_y_mesh, na::Vector3::from_element(1.0_f32)),
            bolt_c: window.add_trimesh(bolt_c_mesh, na::Vector3::from_element(1.0_f32)),
            bolt_x: window.add_trimesh(bolt_x_mesh, na::Vector3::from_element(1.0_f32)),
            bolt_y: window.add_trimesh(bolt_y_mesh, na::Vector3::from_element(1.0_f32)),
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
        
        Box::new(object)
    }
}

impl CalibrationObject for TwoWires {
    fn get_probe(&self) -> Probe {
        Probe::new(vec![
            (self.pos, self.wire_x_shape.clone()),
            (self.pos, self.wire_y_shape.clone()),
            (self.pos, self.bolt_c_shape.clone()),
            (self.pos, self.bolt_x_shape.clone()),
            (self.pos, self.bolt_y_shape.clone()),
        ])
    }

    fn render(&mut self) {
        self.plastic.set_local_transformation(na::convert(self.pos));
        self.wire_x.set_local_transformation(na::convert(self.pos));
        self.wire_y.set_local_transformation(na::convert(self.pos));
        self.bolt_c.set_local_transformation(na::convert(self.pos));
        self.bolt_x.set_local_transformation(na::convert(self.pos));
        self.bolt_y.set_local_transformation(na::convert(self.pos));
    }
}
