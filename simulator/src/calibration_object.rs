use crate::chain::{Transform};

use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use std::path::Path;
//use ncollide3d::bounding_volume::bounding_volume::BoundingVolume;
//use ncollide3d::shape::{Compound, Cuboid, ShapeHandle};

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

    pub fn render(&mut self, _window: &mut Window, _show_collision_bbox: bool) {
        let pos = Transform::translation(0.52, 0.26, 0.0);

        self.plastic.set_local_transformation(pos);
        self.metal.set_local_transformation(pos);
    }
}
