use std::io::{Read, Write};
use std::net::TcpStream;

use eframe::CreationContext;
use egui::{CentralPanel, ColorImage, Image, TextureHandle};
use egui_inspect::EguiInspect;
use webcam_viewer::yuyv2rgb::yuv422_to_rgb24;
use webcam_viewer::{make_secret, tprint, HEIGHT, WIDTH};

#[derive(EguiInspect)]
pub struct LoginData {
    address: String,
    secret: String,
}

pub struct StreamData {
    texture: Option<TextureHandle>,
    stream: TcpStream,
    buf: Vec<u8>,
    rgb: Vec<u8>,
}

impl StreamData {
    fn grab_next_frame(&mut self) -> std::io::Result<()> {
        tprint(format!("reading, expecting {}", self.buf.len()));
        self.stream.read_exact(self.buf.as_mut_slice())?;
        tprint("decoding");
        // can do on client or streamer depending on who has more spare compute
        // toggle with feature?
        yuv422_to_rgb24(self.buf.as_slice(), self.rgb.as_mut_slice());
        Ok(())
    }
}

pub enum App {
    Login(LoginData),
    Stream(StreamData),
}

impl App {
    pub fn new(_cc: &CreationContext) -> Self {
        App::Login(LoginData {
            address: "localhost:3333".to_string(),
            secret: Default::default(),
        })
    }

    pub fn try_login(&mut self) -> std::io::Result<()> {
        if let App::Login(LoginData { address, secret }) = self {
            // on web requires either replacing with websockets or building with
            // wasm-unknown-emscripten (not supported by trunk or eframe?)
            let mut stream = TcpStream::connect(address.clone())?;
            tprint("Successfully connected to server");

            let secret_bytes = make_secret(secret.clone());
            stream.write_all(secret_bytes.as_slice())?;

            let mut sd = StreamData {
                texture: None,
                stream,
                buf: vec![0; WIDTH * HEIGHT * 2],
                rgb: vec![0; WIDTH * HEIGHT * 3],
            };
            // grab early to check if secret was accepted
            sd.grab_next_frame()?;
            *self = App::Stream(sd);
        }
        Ok(())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self {
            App::Login(login) => {
                let mut sc = false;
                CentralPanel::default().show(ctx, |ui| {
                    login.inspect_mut("Login", ui);
                    sc = ui.button("apply").clicked();
                });
                if sc {
                    if let Err(_) = self.try_login() {
                        // feedback?
                    };
                }
            }
            App::Stream(stream_data) => {
                let handle: &mut egui::TextureHandle =
                    stream_data.texture.get_or_insert_with(|| {
                        let cimage =
                            ColorImage::from_rgb([WIDTH, HEIGHT], stream_data.rgb.as_slice());
                        ctx.load_texture("frame", cimage, Default::default())
                    });

                tprint("drawing");
                CentralPanel::default().show(ctx, |ui| {
                    let cimage = ColorImage::from_rgb([WIDTH, HEIGHT], stream_data.rgb.as_slice());
                    handle.set(cimage, Default::default());
                    ui.add(Image::new(&*handle).shrink_to_fit());
                });

                ctx.request_repaint();
                stream_data.grab_next_frame().unwrap(); // panics on server closing
            }
        }
    }
}
