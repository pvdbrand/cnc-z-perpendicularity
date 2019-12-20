use crate::chain::{Parameters, Transform, Bounds};
use crate::mpcnc::{Parameter};
use kiss3d::event::{Action, Key, WindowEvent, Modifiers};
use kiss3d::window::Window;
use na::{Point3};

pub fn draw_transform(window: &mut Window, transform: &Transform, size: f64) {
    let origin = na::convert(transform.transform_point(&Point3::origin()));
    let x = na::convert(transform.transform_point(&Point3::new(size, 0.0, 0.0)));
    let y = na::convert(transform.transform_point(&Point3::new(0.0, size, 0.0)));
    let z = na::convert(transform.transform_point(&Point3::new(0.0, 0.0, size)));

    window.draw_line(&origin, &x, &Point3::new(1.0, 0.0, 0.0));
    window.draw_line(&origin, &y, &Point3::new(0.0, 1.0, 0.0));
    window.draw_line(&origin, &z, &Point3::new(0.0, 0.0, 1.0));
}

pub fn update_parameter(mods: Modifiers, parameters: &mut Parameters<Parameter>, param: Parameter, delta: f64) {
    let mut d = delta;

    if mods.contains(Modifiers::Shift) { d = d / 10.0; }
    if mods.contains(Modifiers::Control) { d = d / 10.0; }
    if mods.contains(Modifiers::Alt) { d = d / 10.0; }

    parameters[param] = param.bounded(parameters[param] + d);
}

pub fn handle_events(window: &mut Window, parameters: &mut Parameters<Parameter>, keyboard_control: bool) {
    let dpos = 0.005;
    let dangle = 1.0_f64.to_radians();

    for event in window.events().iter() {
        if keyboard_control {
            match event.value {
                WindowEvent::Key(key, Action::Press, mods) => match key {
                    Key::Up => update_parameter(mods, parameters, Parameter::Y, dpos),
                    Key::Down => update_parameter(mods, parameters, Parameter::Y, -dpos),
                    Key::Left => update_parameter(mods, parameters, Parameter::X, -dpos),
                    Key::Right => update_parameter(mods, parameters, Parameter::X, dpos),
                    Key::A => update_parameter(mods, parameters, Parameter::Z, dpos),
                    Key::Z => update_parameter(mods, parameters, Parameter::Z, -dpos),
                    Key::S => update_parameter(mods, parameters, Parameter::Spindle, dangle),
                    Key::X => update_parameter(mods, parameters, Parameter::Spindle, -dangle),

                    Key::D => update_parameter(mods, parameters, Parameter::ZAxisX, dangle),
                    Key::C => update_parameter(mods, parameters, Parameter::ZAxisX, -dangle),
                    Key::F => update_parameter(mods, parameters, Parameter::ZAxisY, dangle),
                    Key::V => update_parameter(mods, parameters, Parameter::ZAxisY, -dangle),

                    Key::G => update_parameter(mods, parameters, Parameter::SpindleX, dangle),
                    Key::B => update_parameter(mods, parameters, Parameter::SpindleX, -dangle),
                    Key::H => update_parameter(mods, parameters, Parameter::SpindleY, dangle),
                    Key::N => update_parameter(mods, parameters, Parameter::SpindleY, -dangle),

                    Key::J => update_parameter(mods, parameters, Parameter::EndmillX, dangle),
                    Key::M => update_parameter(mods, parameters, Parameter::EndmillX, -dangle),
                    Key::K => update_parameter(mods, parameters, Parameter::EndmillY, dangle),
                    Key::Comma => update_parameter(mods, parameters, Parameter::EndmillY, -dangle),

                    Key::L => update_parameter(mods, parameters, Parameter::EndmillOffset, dpos / 10.0),
                    Key::Period => update_parameter(mods, parameters, Parameter::EndmillOffset, -dpos / 10.0),

                    _ => {}
                },
                _ => {}
            }
        }
    }
}
