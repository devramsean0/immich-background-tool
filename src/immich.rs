use anyhow::Error;
use rand::Rng;
use reqwest::StatusCode;
use serde::Deserialize;
use std::fs;

pub fn get_image_from_immich(
    client: reqwest::blocking::Client,
    base_path: String,
) -> anyhow::Result<String> {
    let immich_base_path = std::env::var("IMMICH_ENDPOINT")?;
    let immich_album_id = std::env::var("IMMICH_ALBUM")?;

    let album = client
        .get(format!("{immich_base_path}/api/albums/{immich_album_id}"))
        .send()
        .unwrap();
    match album.status() {
        StatusCode::OK => {
            let res = album.json::<ImmichAlbumGetOK>()?;

            let mut continue_looping = true;
            let mut final_path = String::new();
            while continue_looping {
                println!("Assets in Album: {}", res.asset_count);
                if res.assets.is_empty() {
                    return Err(Error::msg("Album contains no assets"));
                }
                let mut rng = rand::rng();
                let idx = rng.random_range(0..res.assets.len());
                println!("Picked asset num {}", idx);
                let asset = &res.assets[idx];

                if asset.asset_type != ImmichAssetType::IMAGE {
                    println!("Asset not image");
                    continue;
                }

                continue_looping = false;

                let path = format!("{}/{}", base_path, asset.original_file_name);
                if fs::exists(path.clone())? {
                    println!("Asset already exists, skipping download");
                    return Ok(path);
                } else {
                    println!("Downloading asset");
                    let raw = client
                        .get(format!(
                            "{immich_base_path}/api/assets/{}/original",
                            asset.id
                        ))
                        .send()?
                        .bytes()?;
                    fs::write(path.clone(), raw)?;
                    println!("Downloaded asset to {path}");
                    final_path = path;
                }
            }

            //Ok(asset.id.clone())
            Ok(final_path)
        }
        _ => {
            let res = album.json::<ImmichRequestBad>()?;
            return Err(Error::msg(format!(
                "{}: {}\n(correlation ID: {})",
                res.status_code,
                res.message.join("\n"),
                res.correlation_id
            )));
        }
    }
}

#[derive(Deserialize, PartialEq)]
enum ImmichAssetType {
    IMAGE,
    VIDEO,
    AUDIO,
    OTHER,
}

#[derive(Deserialize)]
struct ImmichAlbumGetOK {
    #[serde(rename = "assetCount")]
    asset_count: i64,
    assets: Vec<ImmichAlbumAsset>,
}

#[derive(Deserialize)]
struct ImmichRequestBad {
    message: Vec<String>,
    error: String,
    #[serde(rename = "statusCode")]
    status_code: i64,
    #[serde(rename = "correlationId")]
    correlation_id: String,
}

#[derive(Deserialize)]
struct ImmichAlbumAsset {
    id: String,
    checksum: String,
    #[serde(rename = "originalFileName")]
    original_file_name: String,
    #[serde(rename = "type")]
    asset_type: ImmichAssetType,
}
