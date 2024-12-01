use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read, Write},
    net::{TcpStream, UdpSocket},
    time::{self, Duration},
};

use crate::todo::Todo;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SyncState {
    pub last_sync_at: NaiveDateTime,
    pub last_save_at: NaiveDateTime,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum SyncAction {
    //TODO
    #[default]
    Init,
    NoChange,
    Upload,
    Download,
}

pub fn sync_app_data(
    mut local_sync_state: SyncState,
    local_todo_list: Vec<Todo>,
) -> io::Result<Option<(SyncState, Vec<Todo>)>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    socket.send_to(b"yuri", "255.255.255.255:23333")?;
    let mut buf = [0; 10];
    let start_time = time::Instant::now();
    while time::Instant::now().duration_since(start_time) < Duration::from_secs(5) {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                if &buf[..amt] == b"lily" {
                    let mut stream = TcpStream::connect(src)?;
                    let local_todo_list_raw = serde_json::to_vec(&local_todo_list)?;
                    let local_sync_state_raw = serde_json::to_vec(&local_sync_state)?;
                    stream.write_all(&local_todo_list_raw)?;
                    stream.write_all(b"----")?;
                    stream.write_all(&local_sync_state_raw)?;
                    stream.shutdown(std::net::Shutdown::Write)?;
                    let mut data = vec![];
                    let mut buf = [0; 1024];
                    loop {
                        let amt = stream.read(&mut buf)?;
                        if amt == 0 {
                            break;
                        }
                        data.extend_from_slice(&buf[..amt]);
                    }
                    if &data[..6] == b"synced" {
                        local_sync_state.last_sync_at = chrono::Local::now().naive_local();
                        return Ok(Some((local_sync_state, local_todo_list)));
                    } else {
                        let index = data.windows(4).position(|sep| sep == b"----").unwrap();
                        let sync_state_raw = &data.split_off(index + 4);
                        let todo_list_raw = &data[..index];
                        let sync_state = serde_json::from_slice::<SyncState>(&sync_state_raw)?;
                        let todo_list = serde_json::from_slice::<Vec<Todo>>(&todo_list_raw)?;
                        return Ok(Some((sync_state, todo_list)));
                    }
                }
            }
            Err(_) => continue,
        }
    }
    return Ok(None);
}
