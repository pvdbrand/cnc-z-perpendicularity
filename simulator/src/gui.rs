use crate::chain::{Parameters, Transform, Bounds};
use crate::mpcnc::{Parameter};
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::window::Window;
use na::{Point3};
use ncollide3d::bounding_volume::aabb::AABB;

pub fn draw_transform(window: &mut Window, transform: &Transform, size: f32) {
    let origin = transform.transform_point(&Point3::origin());
    let x = transform.transform_point(&Point3::new(size, 0.0, 0.0));
    let y = transform.transform_point(&Point3::new(0.0, size, 0.0));
    let z = transform.transform_point(&Point3::new(0.0, 0.0, size));

    window.draw_line(&origin, &x, &Point3::new(1.0, 0.0, 0.0));
    window.draw_line(&origin, &y, &Point3::new(0.0, 1.0, 0.0));
    window.draw_line(&origin, &z, &Point3::new(0.0, 0.0, 1.0));
}

#[allow(dead_code)]
pub fn draw_aabb(window: &mut Window, aabb: &AABB<f32>, color: &Point3<f32>) {
    let mins = aabb.mins();
    let maxs = aabb.maxs();
    let x1 = mins.coords[0];
    let x2 = maxs.coords[0];
    let y1 = mins.coords[1];
    let y2 = maxs.coords[1];
    let z1 = mins.coords[2];
    let z2 = maxs.coords[2];

    let p1 = Point3::new(x1, y1, z1);
    let p2 = Point3::new(x2, y1, z1);
    let p3 = Point3::new(x2, y2, z1);
    let p4 = Point3::new(x1, y2, z1);
    let p5 = Point3::new(x1, y1, z2);
    let p6 = Point3::new(x2, y1, z2);
    let p7 = Point3::new(x2, y2, z2);
    let p8 = Point3::new(x1, y2, z2);

    window.draw_line(&p1, &p2, color);
    window.draw_line(&p2, &p3, color);
    window.draw_line(&p3, &p4, color);
    window.draw_line(&p4, &p1, color);

    window.draw_line(&p5, &p6, color);
    window.draw_line(&p6, &p7, color);
    window.draw_line(&p7, &p8, color);
    window.draw_line(&p8, &p5, color);

    window.draw_line(&p1, &p5, color);
    window.draw_line(&p2, &p6, color);
    window.draw_line(&p3, &p7, color);
    window.draw_line(&p4, &p8, color);
}

pub fn update_parameter(parameters: &mut Parameters<Parameter>, param: Parameter, delta: f32) {
    parameters[param] = param.bounded(parameters[param] + delta);
}

pub fn handle_events(window: &mut Window, parameters: &mut Parameters<Parameter>, keyboard_control: bool) {
    let dpos = 0.005;
    let dangle = 3.0_f32.to_radians();

    for event in window.events().iter() {
        if keyboard_control {
            match event.value {
                WindowEvent::Key(key, Action::Press, _) => match key {
                    Key::Up => update_parameter(parameters, Parameter::Y, dpos),
                    Key::Down => update_parameter(parameters, Parameter::Y, -dpos),
                    Key::Left => update_parameter(parameters, Parameter::X, -dpos),
                    Key::Right => update_parameter(parameters, Parameter::X, dpos),
                    Key::A => update_parameter(parameters, Parameter::Z, dpos),
                    Key::Z => update_parameter(parameters, Parameter::Z, -dpos),
                    Key::S => update_parameter(parameters, Parameter::Spindle, dangle),
                    Key::X => update_parameter(parameters, Parameter::Spindle, -dangle),

                    Key::D => update_parameter(parameters, Parameter::ZAxisX, dangle),
                    Key::C => update_parameter(parameters, Parameter::ZAxisX, -dangle),
                    Key::F => update_parameter(parameters, Parameter::ZAxisY, dangle),
                    Key::V => update_parameter(parameters, Parameter::ZAxisY, -dangle),

                    Key::G => update_parameter(parameters, Parameter::SpindleX, dangle),
                    Key::B => update_parameter(parameters, Parameter::SpindleX, -dangle),
                    Key::H => update_parameter(parameters, Parameter::SpindleY, dangle),
                    Key::N => update_parameter(parameters, Parameter::SpindleY, -dangle),

                    Key::J => update_parameter(parameters, Parameter::EndmillX, dangle),
                    Key::M => update_parameter(parameters, Parameter::EndmillX, -dangle),
                    Key::K => update_parameter(parameters, Parameter::EndmillY, dangle),
                    Key::Comma => update_parameter(parameters, Parameter::EndmillY, -dangle),

                    Key::L => update_parameter(parameters, Parameter::EndmillOffset, dpos / 10.0),
                    Key::Period => update_parameter(parameters, Parameter::EndmillOffset, -dpos / 10.0),

                    _ => {}
                },
                _ => {}
            }
        }
    }
}
