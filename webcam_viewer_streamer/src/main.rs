use clap::Parser;
use std::io;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{prelude::*, Format};
use v4l::{Device, FourCC};

use webcam_viewer_base::{make_secret, tprint, HEIGHT, SECRET_SIZE, WIDTH};

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
    secret_bytes: &Vec<u8>,
) -> io::Result<()> {
    tprint(format!("Serving: {}", tcp_stream.peer_addr().unwrap()));
    let mut data = vec![0; SECRET_SIZE];
    tcp_stream.read_exact(&mut data)?;
    if *data.as_slice() == *secret_bytes {
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

#[derive(Clone, Debug)]
struct Ip {
    addr: [u8; 4],
}

impl From<String> for Ip {
    fn from(value: String) -> Self {
        let values: Vec<u8> = value
            .split(".")
            .map(|s| s.parse().expect("Not (u8) integer value."))
            .collect();
        if values.len() < 4 {
            panic!("Not enough values.");
        }
        Self {
            addr: [values[0], values[1], values[2], values[3]],
        }
    }
}

#[derive(Parser)]
struct MyArgs {
    #[arg(short, long)]
    /// V4l device id, as appears in /dev/video<dev_id>
    dev_id: Option<usize>,
    #[arg(short, long)]
    ip: Option<Ip>,
    #[arg(short, long)]
    port: Option<usize>,
    #[arg(short, long)]
    secret: Option<String>,
}

fn main() {
    let args = MyArgs::parse();

    let dev_id = args.dev_id.unwrap_or(0);
    let port = args.port.unwrap_or(3333);
    let secret_bytes = match args.secret {
        Some(string) => make_secret(string.clone()),
        None => {
            tprint("USING DEFAULT SECRET \"TEST\" (Should change)");
            make_secret("TEST".to_string())
        }
    };

    let mut dev = Device::new(dev_id).expect("Failed to open device");
    // dbg!(dev.enum_formats().unwrap());

    // TODO: Could request in RGB (rather than converting later), but yuyv is more commonly
    // supported?
    let fmt = Format::new(WIDTH as u32, HEIGHT as u32, FourCC::new(b"YUYV"));
    let _fmt = dev.set_format(&fmt).expect("Failed to write format");
    // dbg!(&_fmt);

    let mut vid_stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)
        .expect("Failed to create buffer stream");

    let addr = match args.ip {
        Some(Ip { addr: [a, b, c, d] }) => format!("{a}.{b}.{c}.{d}:{port}"),
        None => format!("127.0.0.1:{port}"),
    };
    let listener = TcpListener::bind(addr.clone()).unwrap();
    tprint(format!(
        "Server listening on {addr}, will serve frames from /dev/video{dev_id}"
    ));
    for tcp_stream in listener.incoming() {
        match tcp_stream {
            Ok(mut tcp_stream) => {
                // no threads, will handle one connection at a time, exit server on panic
                let _ = handle_client(&mut tcp_stream, &mut vid_stream, &secret_bytes);
            }
            Err(e) => {
                tprint(format!("Error: {}", e));
            }
        }
        tprint("Waiting");
    }
}
