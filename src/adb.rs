use std::process::{Child, Command};

use crate::TCP_PORT;

pub fn reverse_port_adb() -> std::io::Result<Child> {
    Command::new("adb")
        .arg("reverse")
        .arg(format!("tcp:{}", TCP_PORT))
        .arg(format!("tcp:{}", TCP_PORT))
        .spawn()
}
