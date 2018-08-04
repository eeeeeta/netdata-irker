extern crate serde;
extern crate serde_json;
extern crate toml;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate structopt;
extern crate failure;

use structopt::StructOpt;
use failure::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    server: String,
    destinations: Vec<String>,
    ping: Option<String>,
    show_hostname: bool
}

#[derive(StructOpt)]
#[allow(dead_code)]
pub struct Arguments {
    #[structopt(short = "c", long = "config")]
    config: String,
    roles: String,       // the roles that should be notified for this event 
    host: String,        // the host generated this event 
    unique_id: String,   // the unique id of this event 
    alarm_id: String,    // the unique id of the alarm that generated this event 
    event_id: String,    // the incremental id of the event, for this alarm id 
    when: String,        // the timestamp this event occurred 
    name: String,        // the name of the alarm, as given in netdata health.d entries 
    chart: String,       // the name of the chart (type.id) 
    family: String,      // the family of the chart 
    status: String,     // the current status : REMOVED, UNITIALIZED, UNDEFINED, CLEAR, WARNING, CRITICAL 
    old_status: String, // the previous status: REMOVED, UNITIALIZED, UNDEFINED, CLEAR, WARNING, CRITICAL 
    value: String,      // the current value of the alarm 
    old_value: String,  // the previous value of the alarm 
    src: String,        // the line number and file the alarm has been configured 
    duration: String,   // the duration in seconds of the previous alarm state 
    non_clear_duration: String, // the total duration in seconds this is/was non-clear 
    units: String,      // the units of the value 
    info: String,       // a short description of the alarm 
    value_string: String,        // friendly value (with units) 
    old_value_string: String, // friendly old value (with units) 
}

#[derive(Serialize)]
pub struct IrkerNotification {
    to: Vec<String>,
    privmsg: String
}

fn s2c(status: &str) -> &'static str {
    match status {
        "CLEAR" => "09",
        "WARNING" => "07",
        "CRITICAL" => "04",
        _ => "15"
    }
}
fn main() -> Result<(), Error> {
    let args = Arguments::from_args();
    if args.roles == "silent" {
        return Ok(());
    }
    let cfg: Config = toml::from_str(&fs::read_to_string(args.config)?)?;
    let mut stream = TcpStream::connect(&cfg.server)?;
    let ping = if let Some(ref p) = cfg.ping {
        format!("{}: ", p)
    }
    else {
        "".into()
    };
    let hostname = if cfg.show_hostname {
        format!("[\x02\x0306{}\x0f] ", args.host)
    }
    else {
        "".into()
    };
    let body = format!(
        "{}{}[\x02\x03{}{}\x0f] \x02{}\x0f for \x02{}\x0f at \x02\x03{}{}\x0f\x0315 -- desc: {} \x0f[from \x02\x03{}{}\x0f ({})]",
        ping, hostname, s2c(&args.status), args.status, args.name, args.chart, s2c(&args.status), args.value_string,
        args.info, s2c(&args.old_status), args.old_status, args.old_value_string
    );
    let notif = IrkerNotification {
        to: cfg.destinations,
        privmsg: body
    };
    serde_json::to_writer(&mut stream, &notif)?;
    stream.write("\n".as_bytes())?;
    stream.flush()?;
    Ok(())
}
