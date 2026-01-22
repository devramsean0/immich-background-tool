use clap::Parser;
use reqwest::header;

mod immich;

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
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/run")),
        "immich-background-tool/images"
    );
    std::fs::create_dir_all(cache_dir_root.clone()).unwrap();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "x-api-key",
        header::HeaderValue::from_str(&std::env::var("IMMICH_API_KEY").unwrap()).unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // try to download a new photo from immich
    let image_path = match immich::get_image_from_immich(client, cache_dir_root) {
        Ok(path) => path,
        Err(e) => {
            // We should instead pick an already downloaded image or the default
            println!("{e}");
            return;
        }
    };
}
