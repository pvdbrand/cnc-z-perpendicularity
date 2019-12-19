extern crate nalgebra as na;

mod chain;
mod gui;
mod mpcnc;
mod calibration_object;

use crate::chain::{Transform};
use crate::mpcnc::{Parameter};

use kiss3d::camera::ArcBall;
use kiss3d::light::Light;
use kiss3d::text::Font;
use kiss3d::window::Window;
use na::{Point2, Point3};
use std::path::Path;
use std::time::Instant;
use clap::{App, Arg};

fn main() {
    let matches = App::new("Simulator")
        .version("0.1.0")
        .arg(Arg::with_name("manual")
            .long("manual")
            .short("m")
            .help("control the MPCNC with the keyboard"))
        .get_matches();

    simulator(matches.is_present("manual"));
}

fn simulator(manual_control: bool) {
    let resources_dir = Path::new("resources");
    let font = Font::default();
    let mut now = Instant::now();

    let mut window = Window::new_with_size("Simulator", 1280, 720);
    let eye = na::Point3::new(0.5, -1.0, 1.0);
    let at = na::Point3::new(0.5, 0.5, 0.0);
    let mut camera = ArcBall::new(eye, at);

    let mut cnc = mpcnc::MPCNC::new(&mut window, &resources_dir);
    let mut calibration_object = calibration_object::CalibrationObject::new(&mut window, &resources_dir);
    let mut parameters = cnc.get_default_parameters();

    window.set_light(Light::StickToCamera);

    while window.render_with_camera(&mut camera) {
        gui::handle_events(&mut window, &mut parameters, manual_control);
        
        let endmill_tip = cnc.get_end_effector_pos(&parameters);
        
        gui::draw_transform(&mut window, &chain::Transform::identity(), 1.0);
        gui::draw_transform(&mut window, &endmill_tip, 0.1);
        cnc.render(&mut window, &parameters, false);
        calibration_object.render();

        let cnc_probe = &cnc.get_probe_collision_shape(&parameters);
        let cal_probe = &calibration_object.get_probe_collision_shape();

        gui::draw_aabb(&mut window, &cnc_probe.aabb(&Transform::identity()), &Point3::new(1.0, 1.0, 1.0));
        gui::draw_aabb(&mut window, &cal_probe.aabb(&Transform::identity()), &Point3::new(1.0, 1.0, 1.0));


        window.draw_text(&format!("Steppers: X = {:7.3}mm, Y = {:7.3}mm, Z = {:7.3}mm, spindle angle = {:5.1} degrees", 
                parameters[Parameter::X] * 1000.0, parameters[Parameter::Y] * 1000.0, parameters[Parameter::Z] * 1000.0,
                parameters[Parameter::Spindle].to_degrees()),
            &Point2::new(0.0, 0.0), 30.0, &font, &Point3::new(1.0, 1.0, 1.0));
        window.draw_text(&format!("End mill: X = {:7.3}mm, Y = {:7.3}mm, Z = {:7.3}mm", 
                endmill_tip.translation.x * 1000.0, endmill_tip.translation.y * 1000.0, endmill_tip.translation.z * 1000.0),
            &Point2::new(0.0, 30.0), 30.0, &font, &Point3::new(1.0, 1.0, 1.0));
        window.draw_text(&format!("Difference: X = {:7.3}mm, Y = {:7.3}mm, Z = {:7.3}mm", 
                (endmill_tip.translation.x - parameters[Parameter::X]) * 1000.0, 
                (endmill_tip.translation.y - parameters[Parameter::Y]) * 1000.0, 
                (endmill_tip.translation.z - parameters[Parameter::Z]) * 1000.0),
            &Point2::new(0.0, 60.0), 30.0, &font, &Point3::new(1.0, 0.5, 0.5));

        let triggered = ncollide3d::query::proximity(&Transform::identity(), &**cnc_probe, &Transform::identity(), &**cal_probe, 0.0) == ncollide3d::query::Proximity::Intersecting;
        window.draw_text(if triggered { "Z probe: TRIGGERED" } else { "Z probe: open" }, &Point2::new(0.0, 90.0), 30.0, &font, &Point3::new(1.0, 1.0, 1.0));

        let fps = 1.0 / (now.elapsed().as_nanos() as f64 / 1e9_f64);
        now = Instant::now();
        window.draw_text(&format!("FPS: {:.0}", fps.round()), &Point2::new(0.0, 120.0), 30.0, &font, &Point3::new(0.5, 0.5, 0.5));
    }
}
