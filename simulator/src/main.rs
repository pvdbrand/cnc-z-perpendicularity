extern crate nalgebra as na;

mod chain;
mod gui;
mod mpcnc;
mod calibration_object;
mod probe;
mod gcode;

use crate::chain::{Parameters};
use crate::mpcnc::{MPCNC, Parameter};
use crate::calibration_object::CalibrationObject;
use crate::gcode::GCode;

use kiss3d::camera::ArcBall;
use kiss3d::light::Light;
use kiss3d::text::Font;
use kiss3d::window::Window;
use na::{Point2, Point3};
use std::path::Path;
use std::time::Instant;
use clap::{App, Arg};
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;

fn main() {
    let matches = App::new("Simulator")
        .version("0.1.0")
        .arg(Arg::with_name("no-keyboard")
            .long("no-keyboard")
            .short("k")
            .help("disable keyboard controls"))
        .arg(Arg::with_name("fast")
            .long("fast")
            .short("f")
            .help("process gcode as fast as possible without updating the GUI between lines"))
        .get_matches();

    simulator(!matches.is_present("no-keyboard"), matches.is_present("fast"));
}

fn simulator(manual_control: bool, fast: bool) {
    let resources_dir = Path::new("resources");
    let font = Font::default();
    let mut now = Instant::now();
    let stdin_channel = spawn_stdin_channel();
    let mut gcode = gcode::GCode::new();

    let mut window = Window::new_with_size("Simulator", 1280, 720);
    let eye = na::Point3::new(0.5, -1.0, 1.0);
    let at = na::Point3::new(0.5, 0.5, 0.0);
    let mut camera = ArcBall::new(eye, at);

    let mut cnc = mpcnc::MPCNC::new(&mut window, &resources_dir);
    //let mut calibration_object = calibration_object::TwoWires::new(&mut window, &resources_dir);
    let mut calibration_object = calibration_object::FeelerGauge::new(&mut window, &resources_dir);
    let mut parameters = cnc.get_default_parameters();

    window.set_light(Light::StickToCamera);

    while window.render_with_camera(&mut camera) {
        gui::handle_events(&mut window, &mut parameters, manual_control);
        handle_gcode(&stdin_channel, &mut gcode, &mut parameters, &cnc, &calibration_object, fast);

        let endmill_tip = cnc.get_end_effector_pos(&parameters);
        
        gui::draw_transform(&mut window, &chain::Transform::identity(), 1.0);
        gui::draw_transform(&mut window, &endmill_tip, 0.1);
        cnc.render(&mut window, &parameters, false);
        calibration_object.render();

        let cnc_probe = &cnc.get_probe(&parameters);
        let cal_probe = &calibration_object.get_probe();
        let triggered = cnc_probe.is_touching(cal_probe);

        window.draw_text(&format!("Workspace: X = {:7.3}mm, Y = {:7.3}mm, Z = {:7.3}mm", 
                gcode.get_workspace_position(&parameters).x * 1000.0, 
                gcode.get_workspace_position(&parameters).y * 1000.0,
                gcode.get_workspace_position(&parameters).z * 1000.0),
            &Point2::new(0.0, 0.0), 30.0, &font, &Point3::new(1.0, 1.0, 1.0));

        window.draw_text(&format!("Steppers: X = {:7.3}mm, Y = {:7.3}mm, Z = {:7.3}mm, spindle angle = {:5.1} degrees", 
                parameters[Parameter::X] * 1000.0, parameters[Parameter::Y] * 1000.0, parameters[Parameter::Z] * 1000.0,
                parameters[Parameter::Spindle].to_degrees()),
            &Point2::new(0.0, 30.0), 30.0, &font, &Point3::new(1.0, 1.0, 1.0));

        window.draw_text(&format!("End mill: X = {:7.3}mm, Y = {:7.3}mm, Z = {:7.3}mm", 
                endmill_tip.translation.x * 1000.0, endmill_tip.translation.y * 1000.0, endmill_tip.translation.z * 1000.0),
            &Point2::new(0.0, 60.0), 30.0, &font, &Point3::new(1.0, 1.0, 1.0));

        window.draw_text(&format!("Difference: X = {:7.3}mm, Y = {:7.3}mm, Z = {:7.3}mm", 
                (endmill_tip.translation.x - parameters[Parameter::X]) * 1000.0, 
                (endmill_tip.translation.y - parameters[Parameter::Y]) * 1000.0, 
                (endmill_tip.translation.z - parameters[Parameter::Z]) * 1000.0),
            &Point2::new(0.0, 90.0), 30.0, &font, &Point3::new(1.0, 0.5, 0.5));


        window.draw_text(if triggered { "Z probe: TRIGGERED" } else { "Z probe: open" }, &Point2::new(0.0, 120.0), 30.0, &font, &Point3::new(1.0, 1.0, 1.0));

        let fps = 1.0 / (now.elapsed().as_nanos() as f64 / 1e9_f64);
        now = Instant::now();
        window.draw_text(&format!("FPS: {:.0}", fps.round()), &Point2::new(0.0, 150.0), 30.0, &font, &Point3::new(0.5, 0.5, 0.5));
    }
}

fn handle_gcode(stdin_channel: &Receiver<String>, gcode: &mut GCode, parameters: &mut Parameters<Parameter>, cnc: &MPCNC, calibration_object: &Box<dyn CalibrationObject>, fast: bool) -> bool {
    let mut timeout = std::time::Duration::from_millis(0);

    loop {
        match stdin_channel.recv_timeout(timeout) {
            Ok(line) => {
                gcode.parse(line, parameters, cnc, calibration_object);
                if fast {
                    timeout = std::time::Duration::from_millis(500);
                }
            },
            Err(RecvTimeoutError::Timeout) => return true,
            Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        loop {
            let mut buffer = String::new();
            match io::stdin().read_line(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break; // EOF
                    } else {
                        tx.send(buffer).unwrap();
                    }
                }
                Err(err) => {
                    println!("Error while trying to read from standard input: {}", err); 
                    break;
                }
            }
        }
        drop(tx);
    });

    rx
}
