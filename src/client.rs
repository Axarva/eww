use std::process::Stdio;

use crate::opts::{self, ActionClientOnly};
use anyhow::*;
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

pub fn handle_client_only_action(action: ActionClientOnly) -> Result<()> {
    match action {
        ActionClientOnly::Logs => {
            std::process::Command::new("tail")
                .args(["-f", crate::LOG_FILE.to_string_lossy().as_ref()].iter())
                .stdin(Stdio::null())
                .spawn()?
                .wait()?;
        }
    }
    Ok(())
}

pub fn forward_command_to_server(mut stream: UnixStream, action: opts::ActionWithServer) -> Result<()> {
    log::info!("Forwarding options to server");
    stream
        .set_nonblocking(false)
        .context("Failed to set stream to non-blocking")?;

    let message_bytes = bincode::serialize(&action)?;

    stream
        .write(&(message_bytes.len() as u32).to_be_bytes())
        .context("Failed to send command size header to IPC stream")?;

    stream
        .write_all(&message_bytes)
        .context("Failed to write command to IPC stream")?;

    let mut buf = String::new();
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(100)))
        .context("Failed to set read timeout")?;
    stream
        .read_to_string(&mut buf)
        .context("Error reading response from server")?;
    if !buf.is_empty() {
        println!("{}", buf);
    }
    Ok(())
}
