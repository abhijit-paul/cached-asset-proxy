use crate::errors::AppError;
use crate::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::io::Read;

fn not_found(err: impl std::error::Error) -> AppError {
    error!("Failed to decode asset response {}", err);
    AppError::AssetNotFound
}

pub async fn get_asset_definition(
    asset_id: &str,
    version: u64,
    client: Client,
    config: &Config,
) -> Result<warp::hyper::body::Bytes, AppError> {
    let host = &config.assets_url;

    let url = format!(
        "{host}/getAssetDefinition/{asset_id}",
        host = host,
        asset_id = asset_id
    );
    let params = [("version", &*version.to_string())];

    println!("*************URL************ :: {}", url);

    let internal_err = |err| {
        error!("Failed to decode asset response {:?}", err);
        AppError::FailedToGetAsset
    };

    info!("url: {}", &url);
    client
        .get(&url)
        .query(&params)
        .send()
        .await
        .map_err(internal_err)?
        .bytes()
        .await
        .map_err(not_found)
}

async fn get(
    asset_id: &str,
    version: u64,
    file: &str,
    client: Client,
    config: &Config,
) -> Result<warp::hyper::body::Bytes, AppError> {
    let host = &config.assets_url;
    let url = format!("{host}/getAsset?version={version}", host = host);

    let body = [
        ("asset_id", asset_id),
        ("version", &version.to_string()),
        ("files", file),
    ];

    client
        .post(&url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .form(&body)
        .send()
        .await
        .map_err(not_found)?
        .bytes()
        .await
        .map_err(not_found)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Asset(HashMap<String, AssetInfo>);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct AssetInfo {
    #[serde(rename = "AssetPath")]
    pub path: String,
}

pub async fn assetmap(
    asset_id: &str,
    version: u64,
    client: Client,
    config: &Config,
) -> Result<String, AppError> {
    let bytes = get_asset_definition(asset_id, version, client, config)
        .await
        .map_err(not_found)?;

    let content_type = tree_magic::from_u8(&bytes);
    info!("asset content-type: {:?}", content_type);

    let reader = std::io::Cursor::new(bytes.to_vec());

    let mut zip = zip::ZipArchive::new(reader).map_err(asset_error)?;

    let mut main_contents = {
        let mut main_json_file = zip.by_name("asset_config/main.json").map_err(asset_error)?;

        let mut main_contents = String::new();
        main_json_file.read_to_string(&mut main_contents)?;
        main_contents
    };

    let asset = {
        let file = zip
            .by_name("asset_config/main.assets")
            .map_err(asset_error)?;
        let asset: Asset = serde_json::from_reader(file).map_err(AppError::from)?;

        asset
    };

    asset.0.iter().for_each(|(key, asset_info)| {
        main_contents = main_contents.replace(key, &asset_info.path);
    });

    Ok(main_contents)
}

fn asset_error(err: impl std::error::Error) -> AppError {
    error!("Failed to decode asset response {}", err);
    AppError::Asset(err.to_string())
}

pub async fn get_asset(
    config: &Config,
    client: Client,
    address: &str,
    version: u64,
    asset_file_name: &str,
) -> Result<Vec<u8>, AppError> {
    let bytes = get(address, version, asset_file_name, client, config)
        .await
        .map_err(not_found)?;

    let reader = std::io::Cursor::new(bytes.to_vec());
    let mut zip = zip::ZipArchive::new(reader).map_err(not_found)?;

    let mut file = zip.by_name(asset_file_name).map_err(not_found)?;

    let mut bytes: Vec<u8> = vec![];
    file.read_to_end(&mut bytes).map_err(asset_error)?;

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    use httpmock::prelude::*;

    fn get_config() -> crate::config::Config {
        env_logger::try_init().ok();

        crate::config::Config::new().expect("unable to load config")
    }

    #[tokio::test]
    async fn test_assetmap() {
        let asset_id = "123e4567-e89b-12d3-a456-426614174000";
        let version = 2;

        let mut config = get_config();

        let asset = MockServer::start();
        config.assets_url = asset.base_url();

        let asset_mock = asset.mock(|when, then| {
            when.method(GET)
                .path(format!("/getAssetDefinition/{}", asset_id))
                .query_param("version", &*version.to_string());

            then.status(200).body_from_file("fixtures/asset_config.zip");
        });

        let client = reqwest::Client::builder()
            .gzip(true)
            .build()
            .expect("could not build http client");

        let expected = "{\"uid\": 10001,\"name\": \"Asset Config\",\"children\": [{\"uid\": 10002,\"className\": \"Scene\",\"userData\": {\"displayName\": \"Scene 1\"},\"name\": \"default\"}]}";

        let assetmap = assetmap(&asset_id.to_string(), version, client, &config).await;

        // assert_eq!(assetmap.is_ok(), true);
        assert_eq!(expected, assetmap.unwrap());

        asset_mock.assert();
        asset_mock.hits();
    }
}
