use crate::pb::sf::substreams::v1::Package;
use anyhow::{format_err, Context, Error};
use lazy_static::lazy_static;
use prost::Message;
use regex::Regex;
use semver::Version;

lazy_static! {
    static ref MODULE_NAME_REGEXP: Regex = Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]{0,63})$").unwrap();
}

const REGISTRY_URL: &str = "https://spkg.io";

pub async fn read_package(input: &str) -> Result<Package, Error> {
    let mut mutable_input = input.to_string();

    let val = parse_standard_package_and_version(input);
    if let Ok(package_and_version) = val {
        mutable_input = format!(
            "{}/v1/packages/{}/{}",
            REGISTRY_URL, package_and_version.0, package_and_version.1
        );
    }

    if mutable_input.starts_with("http") {
        return read_http_package(&mutable_input).await;
    }

    // Assume it's a local file
    let content = std::fs::read(&mutable_input)
        .context(format_err!("read package from file '{}'", mutable_input))?;
    Package::decode(content.as_ref()).context("decode command")
}

async fn read_http_package(input: &str) -> Result<Package, Error> {
    let body = reqwest::get(input).await?.bytes().await?;
    Package::decode(body).context("decode command")
}

fn parse_standard_package_and_version(input: &str) -> Result<(String, String), Error> {
    let parts: Vec<&str> = input.split('@').collect();
    if parts.len() > 2 {
        return Err(format_err!(
            "package name: {} does not follow the convention of <package>@<version>",
            input
        ));
    }

    let package_name = parts[0].to_string();
    if !MODULE_NAME_REGEXP.is_match(&package_name) {
        return Err(format_err!(
            "package name {} does not match regexp {}",
            package_name,
            MODULE_NAME_REGEXP.as_str()
        ));
    }

    if parts.len() == 1
        || parts
            .get(1)
            .map_or(true, |v| v.is_empty() || *v == "latest")
    {
        return Ok((package_name, "latest".to_string()));
    }

    let version = parts[1];
    if !is_valid_version(&version.replace("v", "")) {
        return Err(format_err!(
            "version '{}' is not valid Semver format",
            version
        ));
    }

    Ok((package_name, version.to_string()))
}

fn is_valid_version(version: &str) -> bool {
    Version::parse(version).is_ok()
}
