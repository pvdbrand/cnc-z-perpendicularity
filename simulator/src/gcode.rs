use crate::chain::{Vec3, Parameters, Bounds};
use crate::mpcnc::{MPCNC, Parameter};
use crate::calibration_object::CalibrationObject;

type Field = Option<Option<f64>>;

pub struct GCode {
    origin: Vec3,
}

impl GCode {
    pub fn new() -> GCode {
        GCode { origin: Vec3::new(0.0, 0.0, 0.0) }
    }

    pub fn parse(&mut self, line: String, parameters: &mut Parameters<Parameter>, cnc: &MPCNC, calibration_object: &Box<dyn CalibrationObject>) {
        let line = line.split(";").collect::<Vec<&str>>()[0];
        let line = line.split("(").collect::<Vec<&str>>()[0];
        let line = line.split("#").collect::<Vec<&str>>()[0];
        let line = line.trim();
        
        let fields = line.split(" ").collect::<Vec<&str>>();
        let a = self.parse_field(&fields, "A");
        let b = self.parse_field(&fields, "B");
        let o = self.parse_field(&fields, "O");
        let r = self.parse_field(&fields, "R");

        let x = self.parse_field(&fields, "X");
        let y = self.parse_field(&fields, "Y");
        let z = self.parse_field(&fields, "Z");

        let has_x = x.is_some();
        let has_y = y.is_some();
        let has_z = z.is_some();

        let x = if let Some(Some(x)) = x { x / 1000.0 } else { parameters[Parameter::X] - self.origin.x };
        let y = if let Some(Some(y)) = y { y / 1000.0 } else { parameters[Parameter::Y] - self.origin.y };
        let z = if let Some(Some(z)) = z { z / 1000.0 } else { parameters[Parameter::Z] - self.origin.z };

        match fields[0] {
            "G0" | "G1" => self.go_to(x, y, z, parameters),

            "G28" => self.home(has_x, has_y, has_z, parameters, cnc, calibration_object),
            "G38.2" => self.probe_towards(x, y, z, parameters, cnc, calibration_object),
            "G38.8" => self.rotate_arm(x, y, z, true, parameters, cnc, calibration_object),
            "G38.9" => self.rotate_arm(x, y, z, false, parameters, cnc, calibration_object),
            "G92" => self.set_position(x, y, z, parameters),
            "M114" => self.get_position(parameters),
            "M119" => self.endstops(parameters, cnc, calibration_object),

            "M800" => self.set_z_axis(a, b, parameters),
            "M801" => self.set_spindle(a, b, r, parameters),
            "M802" => self.set_endmill(a, b, o, parameters),

            "G90" => self.ok(),
            "M17" => self.ok(),
            "M18" => self.ok(),
            "M110" => self.ok(),
            "M400" => self.ok(),
            
            "" => {},
            _ => println!("error:unknown gcode command: {}", line),
        }
    }

    pub fn get_workspace_position(&self, parameters: &Parameters<Parameter>) -> Vec3 {
        Vec3::new(
            parameters[Parameter::X] - self.origin.x,
            parameters[Parameter::Y] - self.origin.y,
            parameters[Parameter::Z] - self.origin.z,
        )
    }

    fn go_to(&self, x: f64, y: f64, z: f64, parameters: &mut Parameters<Parameter>) {
        parameters[Parameter::X] = x + self.origin.x;
        parameters[Parameter::Y] = y + self.origin.y;
        parameters[Parameter::Z] = z + self.origin.z;
        self.ok();
    }

    fn set_position(&mut self, x: f64, y: f64, z: f64, parameters: &mut Parameters<Parameter>) {
        self.origin = Vec3::new(
            parameters[Parameter::X] - x,
            parameters[Parameter::Y] - y,
            parameters[Parameter::Z] - z,
        );
        self.ok();
    }

    fn set_z_axis(&mut self, a: Field, b: Field, parameters: &mut Parameters<Parameter>) {
        if let Some(Some(a)) = a { parameters[Parameter::ZAxisX] = a.to_radians(); }
        if let Some(Some(b)) = b { parameters[Parameter::ZAxisY] = b.to_radians(); }
        self.ok();
    }

    fn set_spindle(&mut self, a: Field, b: Field, r: Field, parameters: &mut Parameters<Parameter>) {
        if let Some(Some(a)) = a { parameters[Parameter::SpindleX] = a.to_radians(); }
        if let Some(Some(b)) = b { parameters[Parameter::SpindleY] = b.to_radians(); }
        if let Some(Some(r)) = r { parameters[Parameter::Spindle] = r.to_radians(); }
        self.ok();
    }

    fn set_endmill(&mut self, a: Field, b: Field, o: Field, parameters: &mut Parameters<Parameter>) {
        if let Some(Some(a)) = a { parameters[Parameter::EndmillX] = a.to_radians(); }
        if let Some(Some(b)) = b { parameters[Parameter::EndmillY] = b.to_radians(); }
        if let Some(Some(o)) = o { parameters[Parameter::EndmillOffset] = o / 1000.0; }
        self.ok();
    }

    fn get_position(&self, parameters: &mut Parameters<Parameter>) {
        let pos = self.get_workspace_position(parameters);
        println!("X:{:.3} Y:{:.3} Z:{:.3}", pos.x * 1000.0, pos.y * 1000.0, pos.z * 1000.0);
        self.ok();
    }

    fn endstops(&self, parameters: &mut Parameters<Parameter>, cnc: &MPCNC, calibration_object: &Box<dyn CalibrationObject>) {
        let triggered = cnc.get_probe(parameters).is_touching(&calibration_object.get_probe());
        println!("z_min: {}", if triggered { "TRIGGERED" } else { "open" });
        self.ok();
    }

    fn home(&mut self, x: bool, y: bool, z: bool, parameters: &mut Parameters<Parameter>, cnc: &MPCNC, calibration_object: &Box<dyn CalibrationObject>) {
        let pos = self.get_workspace_position(parameters);

        if x || y || !z { 
            println!("error:only G28 Z is supported");
        } else {
            self.probe_towards(pos.x, pos.y, -self.origin.z - 0.050, parameters, cnc, calibration_object);
            self.origin.z = parameters[Parameter::Z];
        }
    }

    fn probe_towards(&self, x: f64, y: f64, z: f64, parameters: &mut Parameters<Parameter>, cnc: &MPCNC, calibration_object: &Box<dyn CalibrationObject>) {
        let movement = Vec3::new(x, y, z) - self.get_workspace_position(parameters);
        let mut toi = cnc.get_probe(parameters).approx_time_of_impact(&calibration_object.get_probe(), &movement);

        let microns = movement.norm() * 1e6;
        let time_per_micron = 1.0 / microns;

        let start_x = parameters[Parameter::X];
        let start_y = parameters[Parameter::Y];
        let start_z = parameters[Parameter::Z];

        // back off until the probe is not touching anymore
        loop {
            let delta = movement * toi;
            parameters[Parameter::X] = start_x + delta.x;
            parameters[Parameter::Y] = start_y + delta.y;
            parameters[Parameter::Z] = start_z + delta.z;

            if !cnc.get_probe(parameters).is_touching(&calibration_object.get_probe()) || toi == 0.0 {
                break;
            }

            toi = (toi - time_per_micron).max(0.0);
        }

        // move until the probe is touching again
        loop {
            let delta = movement * toi;
            parameters[Parameter::X] = start_x + delta.x;
            parameters[Parameter::Y] = start_y + delta.y;
            parameters[Parameter::Z] = start_z + delta.z;
            
            if cnc.get_probe(parameters).is_touching(&calibration_object.get_probe()) || toi == 1.0 {
                break;
            }

            toi = (toi + time_per_micron).min(1.0);
        }

        self.ok();
    }

    fn rotate_arm(&self, x: f64, y: f64, z: f64, clockwise: bool, parameters: &mut Parameters<Parameter>, cnc: &MPCNC, calibration_object: &Box<dyn CalibrationObject>) {
        parameters[Parameter::X] = x + self.origin.x;
        parameters[Parameter::Y] = y + self.origin.y;
        parameters[Parameter::Z] = z + self.origin.z;
        
        assert!(cnc.get_probe(parameters).is_touching(&calibration_object.get_probe()));

        // back off until the probe is not touching anymore
        let delta = 0.0001_f64.atan2(0.150) * if clockwise { 1.0 } else { -1.0 };
        let start_angle = parameters[Parameter::Spindle];
        for i in 0..100 {
            parameters[Parameter::Spindle] = Parameter::Spindle.bounded(start_angle - (i as f64) * delta);

            if !cnc.get_probe(parameters).is_touching(&calibration_object.get_probe()) {
                break;
            }
        }

        // rotate until the probe is touching again
        let delta = 0.000001_f64.atan2(0.150) * if clockwise { 1.0 } else { -1.0 };
        let start_angle = parameters[Parameter::Spindle];
        for i in 0..100 {
            parameters[Parameter::Spindle] = Parameter::Spindle.bounded(start_angle + (i as f64) * delta);
            
            if cnc.get_probe(parameters).is_touching(&calibration_object.get_probe()) {
                self.ok();
                return;
            }
        }

        assert!(false);
    }

    fn ok(&self) {
        println!("ok");
    }

    fn parse_field(&self, fields: &Vec<&str>, name: &str) -> Field {
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
}
