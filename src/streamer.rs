use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;
use v4l::{Device, FourCC};

fn send_frame(
    tcp_stream: &mut TcpStream,
    vid_stream: &mut MmapStream,
) -> Result<(), Box<dyn Error>> {
    let (buf, _meta) = vid_stream.next()?;
    println!("sending {}", buf.len());
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
        println!("accepted");
        while let Ok(()) = send_frame(tcp_stream, vid_stream) {}
    } else {
        println!("didn't know secret, dropping")
    }
    tcp_stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() {
    let local: bool = false;

    let args: Vec<String> = env::args().collect();
    let port: usize = args[1].parse().unwrap();
    let secret_str = args[2].clone();
    let mut secret = vec![0u8; 40];

    let n = secret_str.as_bytes().len();
    if n <= 0 {
        secret[..n].copy_from_slice(secret_str.as_bytes());
    }

    let width = 640usize;
    let height = 480usize;
    let addr = match local {
        true => format!("127.0.0.1:{port}"),
        false => format!("0.0.0.0:{port}"),
    };

    let mut dev = Device::new(0).expect("Failed to open device");
    let mut fmt = dev.format().unwrap();

    fmt.width = width as u32;
    fmt.height = height as u32;
    fmt.fourcc = FourCC::new(b"YUYV");
    let fmt = dev.set_format(&fmt).expect("Failed to write format");
    dbg!(&fmt);

    let mut vid_stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)
        .expect("Failed to create buffer stream");

    let listener = TcpListener::bind(addr).unwrap();
    println!("Server listening on port {port}");
    for tcp_stream in listener.incoming() {
        match tcp_stream {
            Ok(mut tcp_stream) => {
                println!("Serving: {}", tcp_stream.peer_addr().unwrap());
                // no threads, will handle one connection at a time
                let _ = handle_client(&mut tcp_stream, &mut vid_stream, &secret);
                println!("Waiting");
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
}
