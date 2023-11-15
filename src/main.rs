//! Local test

use eframe::NativeOptions;
use egui::{CentralPanel, ColorImage, Image, TextureHandle};
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{prelude::*, Format};
use v4l::{Device, FourCC};

mod yuyv2rgb;

#[allow(dead_code)]
struct App<'a> {
    dev: Device,
    fmt: Format,
    width: usize,
    height: usize,
    stream: MmapStream<'a>,
    rgb: Vec<u8>,
    texture: Option<TextureHandle>,
}

impl<'a> App<'a> {
    fn new(width: usize, height: usize) -> Self {
        // TODO: Dev select?
        let mut dev = Device::new(0).expect("Failed to open device");
        let mut fmt = dev.format().unwrap();

        fmt.width = width as u32;
        fmt.height = height as u32;
        fmt.fourcc = FourCC::new(b"YUYV");
        let fmt = dev.set_format(&fmt).expect("Failed to write format");
        dbg!(&fmt);

        let stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)
            .expect("Failed to create buffer stream");

        Self {
            dev,
            fmt,
            stream,
            width,
            height,
            rgb: vec![0; height * width * 3],
            texture: None,
        }
    }
}

impl<'a> eframe::App for App<'a> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let cimage = ColorImage::from_rgb([self.width, self.height], self.rgb.as_slice());
        let handle: &mut egui::TextureHandle = self
            .texture
            .get_or_insert_with(|| ctx.load_texture("frame", cimage, Default::default()));

        let (buf, _meta) = self.stream.next().unwrap();
        // println!(
        //     "Buffer size: {}, seq: {}, timestamp: {}",
        //     buf.len(),
        //     meta.sequence,
        //     meta.timestamp
        // );

        yuyv2rgb::yuv422_to_rgb24(buf, &mut self.rgb.as_mut_slice());

        CentralPanel::default().show(ctx, |ui| {
            let cimage = ColorImage::from_rgb([self.width, self.height], self.rgb.as_slice());
            handle.set(cimage, Default::default());
            ui.add(Image::new(&*handle).shrink_to_fit());
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Webcam test",
        NativeOptions::default(),
        Box::new(|_| Box::new(App::new(640, 480))),
    )
}
