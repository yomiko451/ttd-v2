use chrono::{Datelike, Local, NaiveDateTime, Timelike};
use core::fmt;
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Arc,
    thread,
    time::{self, Duration},
};

use crate::{
    app::{SYNC_STATE_PATH, TODO_LIST_PATH},
    todo::{self, Todo},
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SyncState {
    pub id: String,
    pub last_sync_kind: SyncKind,
    pub last_save_at: NaiveDateTime,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum SyncKind {
    //TODO
    #[default]
    Init,
    LocalSave,
    UploadToServer,
    DownloadFromServer,
}

impl Display for SyncKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncKind::UploadToServer => write!(f, "ToServer"),
            SyncKind::DownloadFromServer => write!(f, "FromServer"),
            SyncKind::LocalSave => write!(f, "LocalSave"),
            SyncKind::Init => write!(f, "Init"),
        }
    }
}

pub fn sync_app_data(
    local_sync_state: SyncState,
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
                        return Ok(None);
                    } else {
                        let index = data.windows(4).position(|sep| sep == b"----").unwrap();
                        let sync_state_raw = &data.split_off(index + 4);
                        let todo_list_raw = &data[..index];
                        let mut sync_state = serde_json::from_slice::<SyncState>(&sync_state_raw)?;
                        sync_state.last_sync_kind = SyncKind::DownloadFromServer;
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
