use crossterm::style::Stylize;
use std::{
    io::{Read, Write},
    net::{TcpListener, UdpSocket},
    path::PathBuf,
    sync::LazyLock,
};
use ttd_v2::{SyncState, Todo};

static CURRENT_PATH: LazyLock<PathBuf> = LazyLock::new(|| std::env::current_dir().unwrap());

static SERVER_SYNC_STATE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CURRENT_PATH.join("server_sync_state.json"));

static SERVER_TODO_LIST_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CURRENT_PATH.join("server_todo_list.json"));

fn main() {
    init().unwrap();
    monitor_broadcast().unwrap(); //TODO
}

fn init() -> std::io::Result<()> {
    if !SERVER_SYNC_STATE_PATH.exists() {
        std::fs::File::create(SERVER_SYNC_STATE_PATH.as_path())?;
    }
    if !SERVER_TODO_LIST_PATH.exists() {
        std::fs::File::create(SERVER_TODO_LIST_PATH.as_path())?;
    }
    println!("Server Data Initialized!");
    println!("Sync Server Started!");
    Ok(())
}

fn monitor_broadcast() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:23333")?;
    let mut buf = [0; 10];
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        if &buf[..amt] == b"yuri" {
            socket.send_to(b"lily", src)?;
            let listener = TcpListener::bind(socket.local_addr().unwrap())?;
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut buf = [0; 1024];
                let mut server_sync_state_raw = std::fs::read(SERVER_SYNC_STATE_PATH.as_path())?;
                let mut server_todo_list_raw = std::fs::read(SERVER_TODO_LIST_PATH.as_path())?;
                let mut data = vec![];
                loop {
                    let amt = stream.read(&mut buf)?;
                    if amt == 0 {
                        break;
                    }
                    data.extend_from_slice(&buf[..amt]);
                }
                let index = data.windows(4).position(|sep| sep == b"----").unwrap();
                let sync_state_raw = &data.split_off(index + 4);
                let todo_list_raw = &data[..index];
                let sync_state = serde_json::from_slice::<SyncState>(&sync_state_raw)?;
                println!("sync start!");
                println!(
                    "---local--- last save at: {} last sync at: {}",
                    sync_state.last_save_at.format("%Y-%m-%d %H:%M:%S"),
                    sync_state.last_sync_at.format("%Y-%m-%d %H:%M:%S")
                );
                let mut server_sync_state =
                    if server_sync_state_raw.is_empty() | server_todo_list_raw.is_empty() {
                        SyncState::default()
                    } else {
                        serde_json::from_slice::<SyncState>(&server_sync_state_raw)?
                    };
                println!(
                    "---server--- last save at: {} last sync at: {}",
                    server_sync_state.last_save_at.format("%Y-%m-%d %H:%M:%S"),
                    server_sync_state.last_sync_at.format("%Y-%m-%d %H:%M:%S")
                );
                if server_sync_state.last_save_at <= sync_state.last_save_at {
                    server_sync_state = sync_state;
                    server_todo_list_raw = todo_list_raw.into();
                    stream.write_all(b"synced")?;
                    stream.shutdown(std::net::Shutdown::Both)?;
                } else {
                    server_sync_state.last_sync_at = chrono::Local::now().naive_local();
                    server_sync_state_raw = serde_json::to_vec(&server_sync_state)?;
                    stream.write_all(&server_todo_list_raw)?;
                    stream.write_all(b"----")?;
                    stream.write_all(&server_sync_state_raw)?;
                    stream.shutdown(std::net::Shutdown::Both)?;
                }
                let sync_log_file = std::fs::File::create(SERVER_SYNC_STATE_PATH.as_path())?;
                serde_json::to_writer(sync_log_file, &server_sync_state)?;
                let todo_list_file = std::fs::File::create(SERVER_TODO_LIST_PATH.as_path())?;
                let server_todo_list = serde_json::from_slice::<Vec<Todo>>(&server_todo_list_raw)?;
                serde_json::to_writer(todo_list_file, &server_todo_list)?;
                println!("{}", "sync success!".green());
                println!(
                    "---server--- last save at: {} last sync at: {}",
                    server_sync_state.last_save_at.format("%Y-%m-%d %H:%M:%S"),
                    server_sync_state.last_sync_at.format("%Y-%m-%d %H:%M:%S")
                );
                break;
            }
        }
    }
}
