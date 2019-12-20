use crate::chain::{Transform};

use ncollide3d::shape::{ShapeHandle, ConvexHull};
use ncollide3d::procedural::TriMesh;

pub struct Probe {
    objects: Vec<(Transform, ShapeHandle<f64>)>,
}

impl Probe {
    pub fn new(objects: Vec<(Transform, ShapeHandle<f64>)>) -> Probe {
        Probe { objects }
    }

    pub fn get_cylinder_shape(diameter: f64, height: f64, transform: &Transform) -> (TriMesh<f32>, ShapeHandle<f64>) {
        let mut mesh64 = ncollide3d::procedural::cylinder(diameter, height, 32);
        let mut mesh32 = ncollide3d::procedural::cylinder(diameter as f32, height as f32, if diameter < 0.001 { 4 } else { 32 });

        mesh64.transform_by(transform);
        mesh32.transform_by(&na::convert(*transform));

        let shape = ConvexHull::try_from_points(&mesh64.coords.clone()).expect("Invalid convex shape");
        shape.check_geometry();
        
        (mesh32, ShapeHandle::new(shape))
    }

    pub fn is_touching(&self, other: &Probe) -> bool {
        for (a_transform, a_shape) in &self.objects {
            for (b_transform, b_shape) in &other.objects {
                if ncollide3d::query::proximity(a_transform, &**a_shape, b_transform, &**b_shape, 0.0) == ncollide3d::query::Proximity::Intersecting {
                    return true;
                }
            }
        }

        false
    }
}
