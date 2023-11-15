//! Local test

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

use eframe::NativeOptions;
use egui::{CentralPanel, ColorImage, Image, TextureHandle};

mod yuyv2rgb;

struct App {
    stream: TcpStream,
    texture: Option<TextureHandle>,
    buf: Vec<u8>,
    rgb: Vec<u8>,
}

impl App {
    fn new(addr: String, secret: Vec<u8>) -> Self {
        let mut stream = TcpStream::connect(addr).unwrap();
        println!("Successfully connected to server in port 3333");
        stream.write_all(secret.as_slice()).unwrap();
        stream.flush().unwrap();
        Self {
            stream,
            texture: None,
            buf: vec![0; 640 * 480 * 2],
            rgb: vec![0; 640 * 480 * 3],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let handle: &mut egui::TextureHandle = self.texture.get_or_insert_with(|| {
            let cimage = ColorImage::from_rgb([640, 480], self.rgb.as_slice());
            ctx.load_texture("frame", cimage, Default::default())
        });

        println!("reading, expecting {}", self.buf.len());
        self.stream.read_exact(self.buf.as_mut_slice()).unwrap();
        println!("decoding");
        yuyv2rgb::yuv422_to_rgb24(self.buf.as_slice(), self.rgb.as_mut_slice());
        println!("drawing");

        CentralPanel::default().show(ctx, |ui| {
            let cimage = ColorImage::from_rgb([640, 480], self.rgb.as_slice());
            handle.set(cimage, Default::default());
            ui.add(Image::new(&*handle).shrink_to_fit());
        });
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let addr = args[1].clone();
    let secret_str = args[2].clone();
    let mut secret = vec![0u8; 40];

    let n = secret_str.as_bytes().len();
    if n <= 0 {
        secret[..n].copy_from_slice(secret_str.as_bytes());
    }

    eframe::run_native(
        "Streamed webcam test",
        NativeOptions::default(),
        Box::new(|_| Box::new(App::new(addr, secret))),
    )
}
