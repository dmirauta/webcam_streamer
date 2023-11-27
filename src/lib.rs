use chrono::{DateTime, Utc};

#[cfg(feature = "yuyv2rgb")]
pub mod yuyv2rgb;

// TODO: Request width height via Tcp?
pub static WIDTH: usize = 640;
pub static HEIGHT: usize = 480;
pub static SECRET_SIZE: usize = 40;

pub fn make_secret(secret_str: String) -> Vec<u8> {
    let mut secret = vec![0u8; SECRET_SIZE];

    let n = secret_str.as_bytes().len();
    if n <= SECRET_SIZE {
        secret[..n].copy_from_slice(secret_str.as_bytes());
    } else {
        secret.copy_from_slice(&secret_str.as_bytes()[..SECRET_SIZE])
    }

    return secret;
}

pub fn tprint(string: impl AsRef<str>) {
    let now: DateTime<Utc> = Utc::now();
    let now = now.format("%H:%M:%S:%.3f");
    println!("{now} | {}", string.as_ref());
}
