#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate serialport;

use rocket::State;
use serialport::prelude::*;
use rocket_contrib::json::{Json};
use std::time::Duration;
use std::sync::Mutex;
use rocket_contrib::serve::StaticFiles;

#[derive(Serialize, Deserialize)]
struct AvailableSerialPort {
    name: String,
}

struct Connection {
    connection: Mutex<Option<Box<dyn SerialPort>>>,
}

// #[get("/")]
// fn index() -> &'static str {
//     "Hello, world!"
// }

#[get("/list_ports")]
fn list_ports() -> Json<Result<Vec<AvailableSerialPort>, String>> {
    match serialport::available_ports() {
        Err(e) => Json(Err(e.description)),
        Ok(ports) => Json(Ok(ports.iter().map(|port| AvailableSerialPort { name: port.port_name.clone() }).collect())),
    }
}

#[get("/connect?<port_name>&<baud_rate>")]
fn connect(port_name: String, baud_rate: u32, connection_state: State<Connection>) -> Json<Result<(), String>> {
    let mut connection = connection_state.connection.lock().expect("Lock connection state");
    
    let settings = SerialPortSettings {
        baud_rate: baud_rate,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(10),
    };

    match serialport::open_with_settings(&port_name, &settings) {
        Err(e) => Json(Err(e.description)),
        Ok(conn) => {
            *connection = Some(conn);
            Json(Ok(()))
        }
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![
            // index, 
            list_ports,
            connect
        ])
        .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
        .manage(Connection { connection: Mutex::new(None) })
        .launch();
}
