use crossterm::style::Stylize;
use ttd_v2::{SyncLog, Todo};
use std::{path::PathBuf, sync::LazyLock, io::{self, Read, Write}, net::{TcpListener, UdpSocket}, time::{self, Duration}};

static CURRENT_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| std::env::current_dir().unwrap());
static SERVER_SYNCLOG_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CURRENT_PATH.join("server_sync.json")
);
static SERVER_DATA_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CURRENT_PATH.join("server_store.json")
);

fn main() {
    init();
    monitor_broadcast().unwrap();//TODO
}

fn init() {
    println!("Sync Server Started!");
    if !SERVER_SYNCLOG_PATH.exists() {
        std::fs::File::create(SERVER_SYNCLOG_PATH.as_path()).unwrap();
    }
}

fn monitor_broadcast() -> std::io::Result<()> {
    let mut sync_log = SyncLog::new();
    if let Ok(log) = std::fs::read(SERVER_SYNCLOG_PATH.as_path()) {
            if !log.is_empty() {
                sync_log = serde_json::from_slice(&log).unwrap();
            } 
    };
    let socket = UdpSocket::bind("0.0.0.0:23333")?;
    let mut buf = [0; 10];
    
    let (amt, src) = socket.recv_from(&mut buf)?;
    if &buf[..amt] == b"yuri" {
        socket.send_to(b"lily", src)?;
        let listener = TcpListener::bind(socket.local_addr().unwrap())?;
        let mut data = vec![];
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buf = [0; 1024];
            loop {
                let amt = stream.read(&mut buf)?;
                if amt == 0 {break;}
                data.extend_from_slice(&buf[..amt]);
            }
            let msg = String::from_utf8_lossy(&data);
            for i in msg.split("----") {
                println!("msg: {}", i);
                println!("------------------------");
            }

            // let sync_data: SyncLog = serde_json::from_slice(&buf).unwrap();
            // if sync_data.last_sync > sync_log.last_sync {
            //     stream.write_all(b"need!").unwrap(); //TODO
            //     let file = std::fs::File::create(SERVER_SYNCLOG_PATH.as_path()).unwrap();
            //     serde_json::to_writer(file, &sync_data).unwrap();
            //     println!("{} - {} - {} - {}", "sync success!".green(), sync_data.id, sync_data.version, sync_data.last_sync.format("%Y-%m-%d %H:%M:%S"));
            // }
            //TODO小于就反过来发过去
        }
    }
    Ok(())

}
