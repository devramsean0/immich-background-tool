use anyhow::Error;
use log::{debug, error, info, warn};
use rand::Rng;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::{env, fs, process::exit};

use crate::utils::{self, backoff};

pub async fn get_image_from_immich(client: Client, base_path: String) -> anyhow::Result<String> {
    let immich_base_path = env::var("IMMICH_ENDPOINT")?;
    let immich_album_id = env::var("IMMICH_ALBUM")?;

    let album = match client
        .get(format!("{immich_base_path}/api/albums/{immich_album_id}"))
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!("Error retrieving album data: {e}");
            return Err(Error::msg(format!("Error retreiving Album metadata: {e}")));
        }
    };
    match album.status() {
        StatusCode::OK => {
            let res = album.json::<ImmichAlbumGetOK>().await?;

            let mut continue_looping = true;
            let mut final_path = String::new();
            while continue_looping {
                debug!("Assets in Album: {}", res.asset_count);
                if res.assets.is_empty() {
                    return Err(Error::msg("Album contains no assets"));
                }
                let mut rng = rand::rng();
                let idx = rng.random_range(0..res.assets.len());
                debug!("Picked asset num {}", idx);
                let asset = &res.assets[idx];

                if asset.asset_type != ImmichAssetType::IMAGE {
                    warn!("Asset not image");
                    continue;
                }

                continue_looping = false;

                let path = format!("{}/{}", base_path, asset.original_file_name);
                if fs::exists(path.clone())? {
                    if utils::checksum::check_checksum_of_file(
                        path.clone(),
                        asset.checksum.clone(),
                    )? {
                        debug!("Asset already exists, skipping download");
                        return Ok(path);
                    } else {
                        download_asset(
                            client.clone(),
                            format!("{immich_base_path}/api/assets/{}/original", asset.id),
                            path.clone(),
                            asset.checksum.clone(),
                        )
                        .await?;
                    }
                } else {
                    download_asset(
                        client.clone(),
                        format!("{immich_base_path}/api/assets/{}/original", asset.id),
                        path.clone(),
                        asset.checksum.clone(),
                    )
                    .await?;
                    final_path = path;
                }
            }
            Ok(final_path)
        }
        _ => {
            let res = album.json::<ImmichRequestBad>().await?;
            return Err(Error::msg(format!(
                "{}: {}\n(correlation ID: {})",
                res.status_code,
                res.message.join("\n"),
                res.correlation_id
            )));
        }
    }
}

async fn download_asset(
    client: Client,
    url: String,
    path: String,
    checksum: String,
) -> anyhow::Result<()> {
    info!("Downloading asset");
    let mut tries = 1;
    // exponential backoff counter
    let mut retry_duration: u64 = 0;
    let mut continue_trying = true;

    while (tries <= 5) && continue_trying {
        debug!("Attempt {tries}");
        let res = match client.get(&url).send().await {
            Ok(res) => res,
            Err(e) => {
                error!("Encountered error making download request: {e} marking as failed");
                backoff::backoff_delay(&mut retry_duration, &tries).await;
                tries += 1;
                continue;
            }
        };

        let raw = match res.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Encountered error converting to bytes during download request: {e} marking as failed");
                backoff::backoff_delay(&mut retry_duration, &tries).await;
                tries += 1;
                continue;
            }
        };
        fs::write(path.clone(), raw)?;
        if !utils::checksum::check_checksum_of_file(path.clone(), checksum.clone())? {
            error!("Checksum invalid after download, uuuh. Marking as failed");
            backoff::backoff_delay(&mut retry_duration, &tries).await;
            tries += 1;
            continue;
        }
        info!("Downloaded asset to {path}");
        continue_trying = false;
    }
    return Ok(());
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
