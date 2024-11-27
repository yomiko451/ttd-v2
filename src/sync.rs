use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream, UdpSocket}, sync::Arc, thread, time::{self, Duration}};
use chrono::{Datelike, NaiveDateTime, Timelike};
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};

use crate::{app::{DATA_PATH, SYNCLOG_PATH}, todo::Todo};


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SyncLog {
    pub id: String,
    pub version: usize,
    pub last_sync: NaiveDateTime,
}

impl SyncLog {
    pub fn new() -> Self {
        SyncLog {
            id: "".to_string(),
            version: usize::default(),
            last_sync: chrono::Local::now().naive_local(),
        }
    }
}

pub fn send_broadcast() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    socket.send_to(b"yuri", "255.255.255.255:23333")?;
    let mut sync_log = serde_json::to_vec(&SyncLog::new()).unwrap();
    if let Ok(log) = std::fs::read(SYNCLOG_PATH.as_path()) {
            if !log.is_empty() {
                sync_log = log;
            } 
    };
    let mut buf = [0; 10];
    let start_time = time::Instant::now();
    while time::Instant::now().duration_since(start_time) < Duration::from_secs(15) {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
            if &buf[..amt] == b"lily" {
                let mut stream = TcpStream::connect(src)?;
                let todo_list = std::fs::read(DATA_PATH.as_path()).unwrap();
                      
                stream.write_all(&todo_list)?;
                stream.write_all(b"----")?;
                stream.write_all(&sync_log)?; 
                
                
                //let mut buf = [0; 1024 * 10];
                // let amt = stream.read(&mut buf).unwrap();
                // if amt > 0 { 
                //     println!("{}", amt);
                // }
                

                return Ok(());
            }
            } 
            Err(_) => continue
        }
    }
    return Ok(());
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_broadcast() {
        
    }
}