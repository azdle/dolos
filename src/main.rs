#![feature(slicing_syntax)]

use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::{IoError, IoErrorKind};

fn main() {
    let mut local_socket = match UdpSocket::bind("0.0.0.0:5683") {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind local socket: {}", e),
    };
    let mut remote_socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind remote socket: {}", e),
    };

    local_socket.set_timeout(Some(100)); //ms
    remote_socket.set_timeout(Some(100)); //ms

    let mut buf = [0; 10];
    let mut proxy_src: Option<SocketAddr> = None;
    loop {
        match local_socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                // Send a reply to the socket we received data from
                let buf = buf.slice_to_mut(amt);
                print!(" >-- ");
                u8_to_str(buf);
                remote_socket.send_to(buf, "coap.exosite.com:5683");
                print!(" --> ");
                u8_to_str(buf);
                proxy_src = Some(src);
            },
            Err(IoError{kind: IoErrorKind::TimedOut, ..}) => (),
            Err(e) => println!("couldn't receive a datagram: {}", e)
        }

        match remote_socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                // Send a reply to the socket we received data from
                let buf = buf.slice_to_mut(amt);
                print!(" --< ");
                u8_to_str(buf);
                if proxy_src.is_some() {
                    local_socket.send_to(buf, proxy_src.unwrap());
                    print!(" <-- ");
                    u8_to_str(buf);
                } else {
                    println!("Warning: Received from Remote Without Known Local Source, Dropping");
                }
            },
            Err(IoError{kind: IoErrorKind::TimedOut, ..}) => (),
            Err(e) => println!("couldn't receive a datagram: {}", e),
        }
    }
}

fn u8_to_str(buf: &[u8]){
    for i in buf.iter() {
        print!("0x{:0>2X}, ", i)
    }
    println!("")
}