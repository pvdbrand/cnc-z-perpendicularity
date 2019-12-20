use crate::chain::{Vec3, Parameters};
use crate::mpcnc::{MPCNC, Parameter};
use crate::calibration_object::CalibrationObject;

type Field = Option<Option<f64>>;

pub fn parse(line: String, parameters: &mut Parameters<Parameter>, cnc: &MPCNC, calibration_object: &CalibrationObject) {
    let line = line.split(";").collect::<Vec<&str>>()[0];
    let line = line.split("(").collect::<Vec<&str>>()[0];
    let line = line.split("#").collect::<Vec<&str>>()[0];
    let line = line.trim();
    
    let fields = line.split(" ").collect::<Vec<&str>>();
    let x = parse_field(&fields, "X");
    let y = parse_field(&fields, "Y");
    let z = parse_field(&fields, "Z");

    match fields[0] {
        "G0" | "G1" => g1(x, y, z, parameters),

        "G38.2" => probe_towards(x, y, z, parameters, cnc, calibration_object),
        "G38.4" => {},
        "G92" => {},
        "M114" => {},
        "M119" => {},

        "G90" => {},
        "M17" => {},
        "M18" => {},
        "M110" => {},
        "M400" => {},
        _ => println!("Ignoring unknown gcode command: {}", line),
    }
}

fn g1(x: Field, y: Field, z: Field, parameters: &mut Parameters<Parameter>) {
    if let Some(Some(x)) = x { parameters[Parameter::X] = x / 1000.0; }
    if let Some(Some(y)) = y { parameters[Parameter::Y] = y / 1000.0; }
    if let Some(Some(z)) = z { parameters[Parameter::Z] = z / 1000.0; }
}

fn probe_towards(x: Field, y: Field, z: Field, parameters: &mut Parameters<Parameter>, cnc: &MPCNC, calibration_object: &CalibrationObject) {
    let x = if let Some(Some(x)) = x { x / 1000.0 } else { parameters[Parameter::X] };
    let y = if let Some(Some(y)) = y { y / 1000.0 } else { parameters[Parameter::Y] };
    let z = if let Some(Some(z)) = z { z / 1000.0 } else { parameters[Parameter::Z] };
    
    let movement = Vec3::new(x, y, z) - Vec3::new(parameters[Parameter::X], parameters[Parameter::Y], parameters[Parameter::Z]);
    let delta = cnc.get_probe(parameters).probe_towards(&calibration_object.get_probe(), &movement).unwrap_or(movement);

    parameters[Parameter::X] += delta.x;
    parameters[Parameter::Y] += delta.y;
    parameters[Parameter::Z] += delta.z;
}

fn parse_field(fields: &Vec<&str>, name: &str) -> Field {
    for field in fields {
        if field.starts_with(name) {
            let value = &field[name.len()..];

            if value.len() == 0 {
                return Some(None)
            } else {
                return Some(Some(value.parse().expect("expected floating point value")));
            }
        }
    }

    None
}