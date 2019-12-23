use crate::chain::{Transform, Vec3};

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

    pub fn get_box_shape(x: f64, y: f64, z: f64, transform: &Transform) -> (TriMesh<f32>, ShapeHandle<f64>) {
        let mut mesh64 = ncollide3d::procedural::cuboid(&Vec3::new(x, y, z));
        let mut mesh32 = ncollide3d::procedural::cuboid(&na::Vector3::new(x as f32, y as f32, z as f32));

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

    pub fn approx_time_of_impact(&self, other: &Probe, movement: &Vec3) -> f64 {
        let mut smallest_toi = 1.0;

        for (a_transform, a_shape) in &self.objects {
            for (b_transform, b_shape) in &other.objects {
                let toi = ncollide3d::query::time_of_impact(a_transform, movement, &**a_shape, b_transform, &Vec3::zeros(), &**b_shape, 1.0, 0.0);

                if let Some(time) = toi {
                    if time.toi < smallest_toi {
                        smallest_toi = time.toi;
                    }
                }
            }
        }

        smallest_toi.max(0.0).min(1.0)
    }
/*
    pub fn probe_towards(&self, other: &Probe, movement: &Vec3) -> Vec3 {
        let mut result = movement.clone();

        for (a_transform, a_shape) in &self.objects {
            for (b_transform, b_shape) in &other.objects {
                let toi = ncollide3d::query::time_of_impact(a_transform, movement, &**a_shape, b_transform, &Vec3::zeros(), &**b_shape, 1.0, 0.0);

                if let Some(toi) = toi {
                    let mut delta = movement * toi.toi;

                    for _ in 0..100 {
                        if delta.norm() >= result.norm() {
                            break;
                        }
                        if ncollide3d::query::proximity(&(a_transform * &Transform::translation(delta.x, delta.y, delta.z)), &**a_shape, 
                                                        b_transform, &**b_shape, 0.0) == ncollide3d::query::Proximity::Intersecting {
                            result = delta;
                            break;
                        }
                        delta += movement.normalize() * 1e-6;
                    }
                }
            }
        }

        result
    }
*/
}
