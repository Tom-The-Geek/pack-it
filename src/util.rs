use anyhow::Result;
use colored::Colorize;
use sha1::Digest;
use terminal_size::{terminal_size, Width};

pub const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/Geek202/pack-it)",
);

pub fn print_hello() {
    let hello = " Welcome to pack-it v0.1 ";

    let w = if let Some((Width(w), _)) = terminal_size() { w as usize } else { hello.len() };
    let padding_length = (w / 2) - (hello.len() / 2);
    let mut padding = String::with_capacity(padding_length);
    if padding_length > 0 {
        for _ in 0..padding_length {
            padding += " ";
        }
    }

    println!("{}{}\n", padding, hello.black().bold().on_bright_cyan());
}

pub fn info(message: &str) {
    println!("ℹ️  {}", message.truecolor(0x88, 0x88, 0x88))
}

pub fn error(message: &str) {
    eprintln!("💥 {}", message.red());
}

pub fn warning(message: &str) {
    eprintln!("⚠️  {}", message.yellow())
}

pub fn complete(message: &str) {
    println!("🎉 {}", message.green());
}

pub async fn hash_from_url(url: &str) -> Result<String> {
    let data = reqwest::get(url).await?
        .bytes().await?;

    let digest = sha1::Sha1::digest(&*data);
    Ok(format!("{:02x}", digest))
}
