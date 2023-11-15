use chrono::{DateTime, Utc};

pub fn make_secret(secret_str: String) -> Vec<u8> {
    let mut secret = vec![0u8; 40];

    let n = secret_str.as_bytes().len();
    if n <= 40 {
        secret[..n].copy_from_slice(secret_str.as_bytes());
    } else {
        secret.copy_from_slice(secret_str[..40].as_bytes())
    }

    return secret;
}

pub fn tprint(string: impl AsRef<str>) {
    let now: DateTime<Utc> = Utc::now();
    let now = now.format("%H:%M:%S:%.3f");
    println!("{now} | {}", string.as_ref());
}
