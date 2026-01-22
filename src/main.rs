use std::{env, fs};

use clap::Parser;
use rand::Rng;
use reqwest::header;

mod immich;
mod sway;

#[derive(Parser, Debug)]
#[clap(author = "Sean Outram", version, about)]
/// Application configuration
struct Args {
    #[arg()]
    env_file_path: Option<String>,
}

fn main() {
    let args = Args::parse();

    // Load configuration from env file
    let env_file_path = args.env_file_path.unwrap_or_else(|| String::from(".env"));
    dotenv::from_filename(env_file_path).unwrap();
    let cache_dir_root = format!(
        "{}/{}",
        env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/run")),
        "immich-background-tool/images"
    );
    fs::create_dir_all(cache_dir_root.clone()).unwrap();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "x-api-key",
        header::HeaderValue::from_str(&env::var("IMMICH_API_KEY").unwrap()).unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // try to download a new photo from immich
    let image_path = match immich::get_image_from_immich(client, cache_dir_root.clone()) {
        Ok(path) => path,
        Err(e) => {
            println!("{e}");
            let files: Vec<fs::DirEntry> = fs::read_dir(cache_dir_root)
                .unwrap()
                .filter_map(Result::ok)
                .collect();
            if files.is_empty() {
                println!("{e}");
                return;
            }

            let mut rng = rand::rng();
            let idx = rng.random_range(0..files.len());
            println!("Picked asset num {}", idx);
            let asset = &files[idx];

            asset.path().to_str().unwrap().to_string()
        }
    };

    sway::issue_bg_update(image_path);
}
