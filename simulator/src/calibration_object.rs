use crate::chain::{Transform, Vec3};

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::path::Path;
use ncollide3d::shape::{Shape, Compound, ShapeHandle, Capsule};
use na::{Translation3, UnitQuaternion};

pub struct CalibrationObject {
    plastic: SceneNode,
    metal: SceneNode,
}

impl CalibrationObject {
    pub fn new(window: &mut Window, resources_dir: &Path) -> CalibrationObject {
        let mm = na::Vector3::new(0.001, 0.001, 0.001);

        let mut object = CalibrationObject {
            plastic: window.add_obj(&resources_dir.join("calibration-object-plastic.obj"), resources_dir, mm),
            metal: window.add_obj(&resources_dir.join("calibration-object-metal.obj"), resources_dir, mm),
        };

        object.plastic.set_color(1.0, 1.0, 0.0);
        object.metal.set_color(0.75, 0.75, 0.75);
        
        object
    }

    pub fn get_probe_collision_shape(&self) -> Box<dyn Shape<f32>> {
        let length = 0.04;
        let radius = 0.08 / 1000.0 / 2.0;

        let along_y = Transform::from_parts(
            Translation3::new(-0.0215, -0.010 + length / 2.0, 0.036), 
            UnitQuaternion::identity()
        );
        let along_x = Transform::from_parts(
            Translation3::new(-0.020 + length / 2.0, -0.0115, 0.036), 
            UnitQuaternion::from_axis_angle(&Vec3::z_axis(), 270.0_f32.to_radians())
        );

        Box::new(Compound::new(vec![(
            Transform::translation(0.52, 0.26, 0.0) * along_y,
            ShapeHandle::new(Capsule::new(length / 2.0 - radius, radius)),
        ), (
            Transform::translation(0.52, 0.26, 0.0) * along_x,
            ShapeHandle::new(Capsule::new(length / 2.0 - radius, radius)),
        )]))
    }

    pub fn render(&mut self) {
        let pos = Transform::translation(0.52, 0.26, 0.0);

        self.plastic.set_local_transformation(pos);
        self.metal.set_local_transformation(pos);
    }
}
