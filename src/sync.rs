use std::{io::{self, Read, Write}, net::{IpAddr, TcpStream, UdpSocket}, time::{self, Duration}};



fn send_broadcast() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:23333")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    socket.send_to(b"yuri!", "255.255.255.255:23333")?;
    let mut buf = [0; 10];
    let start_time = time::Instant::now();
    while time::Instant::now().duration_since(start_time) < Duration::from_secs(15) {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let msg = String::from_utf8_lossy(&buf[..amt]);
            if msg == "lily!" {
                let mut stream = TcpStream::connect(src)?;
                stream.write(b"2024-12-12")?;
                let mut buf = vec![];
                stream.read_to_end(&mut buf)?;

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
        assert!(send_broadcast().is_ok());
    }
}