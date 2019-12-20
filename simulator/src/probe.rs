use crate::chain::{Transform, Vec3};

use ncollide3d::shape::{ShapeHandle, ConvexHull};
use ncollide3d::procedural::TriMesh;

pub struct Probe {
    objects: Vec<(Transform, ShapeHandle<f32>)>,
}

impl Probe {
    pub fn new(objects: Vec<(Transform, ShapeHandle<f32>)>) -> Probe {
        Probe { objects }
    }

    pub fn get_cylinder_shape(diameter: f32, height: f32, transform: &Transform) -> (TriMesh<f32>, ShapeHandle<f32>) {
        // let mut trimesh = ncollide3d::procedural::cylinder(diameter, height, 32);
        let mut mesh = ncollide3d::procedural::cuboid(&Vec3::new(diameter, height, diameter));

        mesh.transform_by(transform);

        // let indices = trimesh.flat_indices().iter().map(|x| *x as usize).collect::<Vec<usize>>();
        // let hull = ConvexHull::try_new(trimesh.coords.clone(), &indices[..]).expect("Invalid convex shape");
        let shape = ConvexHull::try_from_points(&mesh.coords.clone()).expect("Invalid convex shape");
        shape.check_geometry();
        
        (mesh, ShapeHandle::new(shape))
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
