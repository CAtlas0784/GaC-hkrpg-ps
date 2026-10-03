use std::collections::HashMap;

use anyhow::{Result, anyhow};
use prost::Message as _;
use proto::GateServer;
use serde::{Deserialize, Serialize};
use tokio::fs;

const DEFAULT_VERSIONS: &str = include_str!("../../../versions.json");

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VersionConfig {
    pub asset_bundle_url: String,
    pub ex_resource_url: String,
    pub lua_url: String,
    pub ifix_url: String,
}

impl VersionConfig {
    const CNPROD_HOST: &str = "prod-gf-cn-dp01.bhsr.com";
    const CNBETA_HOST: &str = "beta-release01-cn.bhsr.com";
    const OSPROD_HOST: &str = "prod-official-asia-dp01.starrails.com";
    const OSBETA_HOST: &str = "beta-release01-asia.starrails.com";

    const PROXY_HOST: &str = "proxy1.neonteam.dev"; // we used this because PS users usually have a proxy enabled

    pub async fn load_hotfix() -> HashMap<String, VersionConfig> {
        const CONFIG_PATH: &str = "versions.json";

        let data = match fs::read_to_string(CONFIG_PATH).await {
            Ok(data) => data,
            Err(_) => {
                fs::write(CONFIG_PATH, DEFAULT_VERSIONS)
                    .await
                    .expect("Failed to create versions file");
                DEFAULT_VERSIONS.to_string()
            }
        };

        match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(_) => {
                tracing::error!("malformed versions.json. replacing it with default one");
                let _ = fs::write(CONFIG_PATH, DEFAULT_VERSIONS).await;
                serde_json::from_str(DEFAULT_VERSIONS).unwrap()
            }
        }
    }

    pub async fn fetch_hotfix(version: &str, dispatch_seed: &str) -> Result<VersionConfig> {
        let (region, branch, _, _) = Self::parse_version_string(version)?;

        let host = match (region.as_str(), branch.as_str()) {
            ("OS", "BETA") => Self::OSBETA_HOST,
            ("OS", "PROD") => Self::OSPROD_HOST,
            ("CN", "BETA") => Self::CNBETA_HOST,
            ("CN", "PROD") => Self::CNPROD_HOST,
            // TODO: Support more host, or use query_dispatch result to determine the urls
            _ => Self::OSBETA_HOST,
        };

        let url = format!(
            "https://{}/{host}/query_gateway?version={}&platform_type=1&language_type=3&dispatch_seed={}&channel_id=1&sub_channel_id=1&is_need_url=1",
            Self::PROXY_HOST,
            version,
            dispatch_seed
        );

        tracing::info!("fetching hotfix: {url}");

        let res = reqwest::get(url).await?.text().await?;

        tracing::info!("raw gateway response: {}", res);

        let bytes = rbase64::decode(&res)?;
        let decoded = GateServer::decode(bytes.as_slice())?;

        if decoded.asset_bundle_url.is_empty() && decoded.ex_resource_url.is_empty() {
            return Err(anyhow::format_err!(
                "asset_bundle_url and ex_resource_url are empty! gateway response: {:?}",
                decoded
            ));
        }

        tracing::info!("{:#?}", decoded);

        Ok(VersionConfig {
            asset_bundle_url: decoded.asset_bundle_url,
            ex_resource_url: decoded.ex_resource_url,
            lua_url: decoded.lua_url,
            ifix_url: decoded.ifix_url,
        })
    }

    fn parse_version_string(s: &str) -> Result<(String, String, String, String)> {
        const BRANCHES: [&str; 7] = ["PREbeta", "BETA", "PROD", "DEV", "PRE", "GM", "CECREATION"];
        const OS: [&str; 3] = ["Android", "Win", "iOS"];

        let region = s
            .get(0..2)
            .ok_or_else(|| anyhow!("version parse failed. reason: invalid region url: {s}"))?;
        let after_region = s.get(2..).unwrap_or(s);

        let branch = BRANCHES
            .iter()
            .find(|&&b| after_region.starts_with(b))
            .map(|&b| b.to_string())
            .ok_or_else(|| anyhow!("version parse failed. reason: invalid branch url: {s}"))?;

        let branch_len = branch.len();
        let after_branch = after_region.get(branch_len..).unwrap_or(after_region);

        let os = OS
            .iter()
            .find(|&&o| after_branch.starts_with(o))
            .map(|&o| o.to_string())
            .ok_or_else(|| anyhow!("version parse failed. reason: invalid os url: {s}"))?;

        let os_len = os.len();
        let version = after_branch
            .get(os_len..)
            .ok_or_else(|| anyhow!("version parse failed. reason: invalid version url: {s}"))?;

        Ok((region.to_string(), branch, os, version.to_string()))
    }
}
