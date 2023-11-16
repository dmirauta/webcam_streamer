use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::{env, io};
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{prelude::*, Format};
use v4l::{Device, FourCC};

use webcam_viewer::{make_secret, tprint};

enum FrameError {
    Vid,
    Tcp,
}

fn send_frame(tcp_stream: &mut TcpStream, vid_stream: &mut MmapStream) -> Result<(), FrameError> {
    let (buf, _meta) = vid_stream.next().map_err(|_| return FrameError::Vid)?;
    tprint(format!("sending {}", buf.len()));
    tcp_stream
        .write_all(buf)
        .map_err(|_| return FrameError::Tcp)?;
    Ok(())
}

fn handle_client(
    tcp_stream: &mut TcpStream,
    vid_stream: &mut MmapStream,
    secret: &Vec<u8>,
) -> io::Result<()> {
    let mut data = vec![0; 40];
    tcp_stream.read_exact(&mut data)?;
    if *data.as_slice() == *secret {
        tprint("accepted");
        loop {
            match send_frame(tcp_stream, vid_stream) {
                Ok(_) => {}
                Err(FrameError::Vid) => panic!("Video stream not available."),

                Err(FrameError::Tcp) => break,
            }
        }
    } else {
        tprint("declined")
    }
    tcp_stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() {
    let local: bool = false;

    let args: Vec<String> = env::args().collect();
    let dev_id: usize = args[1].parse().unwrap();
    let port: usize = args[2].parse().unwrap();
    let secret = make_secret(args[3].clone());
    dbg!(&dev_id, &port);

    let mut dev = Device::new(dev_id).expect("Failed to open device");
    dbg!(dev.enum_formats().unwrap());

    let fmt = Format::new(640, 480, FourCC::new(b"YUYV"));
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
                // no threads, will handle one connection at a time, exit server on panic
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
