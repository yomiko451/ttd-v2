use crossterm::style::Stylize;
use std::{
    io::{Read, Write},
    net::{TcpListener, UdpSocket},
    path::PathBuf,
    sync::LazyLock,
};
use ttd_v2::{SyncKind, SyncState, Todo};

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
                    "---local--- id: {} last sync state: {} last save at: {}",
                    sync_state.id,
                    sync_state.last_sync_kind,
                    sync_state.last_save_at.format("%Y-%m-%d %H:%M:%S")
                );
                let mut server_sync_state =
                    if server_sync_state_raw.is_empty() | server_todo_list_raw.is_empty() {
                        SyncState::default()
                    } else {
                        serde_json::from_slice::<SyncState>(&server_sync_state_raw)?
                    };
                println!(
                    "---server--- id: {} last sync state: {} last save at: {}",
                    server_sync_state.id,
                    server_sync_state.last_sync_kind,
                    server_sync_state.last_save_at.format("%Y-%m-%d %H:%M:%S")
                );
                if (server_sync_state.last_sync_kind == SyncKind::Init)
                    && (sync_state.last_sync_kind == SyncKind::Init)
                {
                    stream.write_all(b"synced")?;
                    stream.shutdown(std::net::Shutdown::Both)?;
                    println!("init! there is no data to sync!");
                    break;
                } else if server_sync_state.last_save_at == sync_state.last_save_at {
                    stream.write_all(b"synced")?;
                    stream.shutdown(std::net::Shutdown::Both)?;
                    println!("same data! there is no need to sync!");
                    break;
                } else if (server_sync_state.last_save_at < sync_state.last_save_at)
                    || (server_sync_state.last_sync_kind == SyncKind::Init)
                {
                    server_sync_state = sync_state;
                    server_sync_state.last_sync_kind = SyncKind::UploadToServer;
                    server_todo_list_raw = todo_list_raw.into();
                    stream.write_all(b"synced")?;
                    stream.shutdown(std::net::Shutdown::Both)?;
                    println!("{}", "upload success!".green());
                } else {
                    server_sync_state.last_sync_kind = SyncKind::DownloadFromServer;
                    server_sync_state_raw = serde_json::to_vec(&server_sync_state)?;
                    stream.write_all(&server_todo_list_raw)?;
                    stream.write_all(b"----")?;
                    stream.write_all(&server_sync_state_raw)?;
                    stream.shutdown(std::net::Shutdown::Both)?;
                    println!("{}", "download success!".green());
                }
                let sync_log_file = std::fs::File::create(SERVER_SYNC_STATE_PATH.as_path())?;
                serde_json::to_writer(sync_log_file, &server_sync_state)?;
                let todo_list_file = std::fs::File::create(SERVER_TODO_LIST_PATH.as_path())?;
                let server_todo_list = serde_json::from_slice::<Vec<Todo>>(&server_todo_list_raw)?;
                serde_json::to_writer(todo_list_file, &server_todo_list)?;
                println!(
                    "---server--- id: {} current sync state: {} last save at: {}",
                    server_sync_state.id,
                    server_sync_state.last_sync_kind,
                    server_sync_state.last_save_at.format("%Y-%m-%d %H:%M:%S")
                );
                break;
            }
        }
    }
}
