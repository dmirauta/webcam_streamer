use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;
use v4l::{Device, FourCC};

use crate::shared::{make_secret, tprint};

mod shared;

fn send_frame(
    tcp_stream: &mut TcpStream,
    vid_stream: &mut MmapStream,
) -> Result<(), Box<dyn Error>> {
    let (buf, _meta) = vid_stream.next()?;
    tprint(format!("sending {}", buf.len()));
    tcp_stream.write_all(buf)?;
    Ok(())
}

fn handle_client(
    tcp_stream: &mut TcpStream,
    vid_stream: &mut MmapStream,
    secret: &Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let mut data = vec![0; 40];
    tcp_stream.read_exact(&mut data)?;
    if *data.as_slice() == *secret {
        tprint("accepted");
        while let Ok(()) = send_frame(tcp_stream, vid_stream) {}
    } else {
        tprint("declined")
    }
    tcp_stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() {
    let local: bool = false;

    let args: Vec<String> = env::args().collect();
    let port: usize = args[1].parse().unwrap();
    let secret = make_secret(args[2].clone());

    let mut dev = Device::new(0).expect("Failed to open device");
    let mut fmt = dev.format().unwrap();

    fmt.width = 640;
    fmt.height = 480;
    fmt.fourcc = FourCC::new(b"YUYV");
    let fmt = dev.set_format(&fmt).expect("Failed to write format");
    dbg!(&fmt);

    let mut vid_stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)
        .expect("Failed to create buffer stream");

    let addr = match local {
        true => format!("127.0.0.1:{port}"),
        false => format!("0.0.0.0:{port}"),
    };
    let listener = TcpListener::bind(addr).unwrap();
    tprint("Server listening on port {port}");
    for tcp_stream in listener.incoming() {
        match tcp_stream {
            Ok(mut tcp_stream) => {
                tprint(format!("Serving: {}", tcp_stream.peer_addr().unwrap()));
                // no threads, will handle one connection at a time
                let _ = handle_client(&mut tcp_stream, &mut vid_stream, &secret);
                tprint("Waiting");
            }
            Err(e) => {
                tprint(format!("Error: {}", e));
                /* connection failed */
            }
        }
    }
}
