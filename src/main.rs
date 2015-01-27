#![feature(slicing_syntax)]
#![feature(plugin)]
#![allow(unstable)]

extern crate "rustc-serialize" as rustc_serialize;

extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;

use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::{IoError, IoErrorKind};

docopt!(Args derive Show, "
Usage: dolos [options] [<srcip>] <srcport> <dstip> <dstport>
       dolos --help

Options:
  -h, --help       Show this message.
  -v, --verbose    Print more information.
",
arg_srcport: u16,
arg_dstport: u16,);

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    println!("Proxying {}:{} to {}:{}.", args.arg_srcip, args.arg_srcport, args.arg_dstip, args.arg_dstport);

    let local_addr = match args.arg_dstip.as_slice() {
        "" => (args.arg_dstip.as_slice(), args.arg_dstport),
        _  => ("0.0.0.0".as_slice(), args.arg_dstport),
    };
    let remote_addr = SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 0 };
    let dest_addr = (args.arg_dstip.as_slice(), args.arg_dstport);

    let mut local_socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind local socket: {}", e),
    };
    let mut remote_socket = match UdpSocket::bind(remote_addr) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind remote socket: {}", e),
    };

    let mut buf = [0; 2048];
    let mut proxy_src: Option<SocketAddr> = None;
    loop {
        // these need to be re-set on every call
        local_socket.set_timeout(Some(100)); //ms
        remote_socket.set_timeout(Some(100)); //ms

        match local_socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                if proxy_src.is_some() && src != proxy_src.unwrap() {
                    println!("WARNING: Received from new src while already bound, dropping.");
                    return
                }
                // Send a reply to the socket we received data from
                let buf = &mut buf[.. amt];
                if proxy_src.is_some() {
                    remote_socket.send_to(buf, dest_addr).ok();
                    print!(" >--> ");
                    print_u8(buf);
                } else {
                    print!(" >--X ");
                    print_u8(buf);
                }
                proxy_src = Some(src);
            },
            Err(IoError{kind: IoErrorKind::TimedOut, ..}) => (),
            Err(e) => println!("couldn't receive a datagram: {}", e)
        }

        match remote_socket.recv_from(&mut buf) {
            Ok((amt, _src)) => {
                // Send a reply to the socket we received data from
                let buf = &mut buf[.. amt];
                if proxy_src.is_some() {
                    local_socket.send_to(buf, proxy_src.unwrap()).ok();
                    print!(" <--< ");
                    print_u8(buf);
                } else {
                    print!(" X--< ");
                    print_u8(buf);
                }
            },
            Err(IoError{kind: IoErrorKind::TimedOut, ..}) => (),
            Err(e) => println!("couldn't receive a datagram: {}", e),
        }
    }
}

fn print_u8(buf: &[u8]){
    for i in buf.iter() {
        print!("0x{:0>2X}, ", i)
    }
    println!("")
}