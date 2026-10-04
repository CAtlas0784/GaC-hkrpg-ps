use std::sync::Arc;

use crate::AppState;
use axum::extract::{Query, State};
use prost::Message;
use proto::{Dispatch, GateServer, RegionInfo};
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::instrument;

pub const QUERY_DISPATCH_ENDPOINT: &str = "/query_dispatch";
pub const QUERY_GATEWAY_ENDPOINT: &str = "/query_gateway";

#[tracing::instrument]
pub async fn query_dispatch() -> String {
    let rsp = Dispatch {
        retcode: 0,
        region_list: vec![RegionInfo {
            name: String::from("GaC hkrpg ps"),
            title: String::from("GaC hkrpg ps"),
            env_type: String::from("9"),
            dispatch_url: String::from("http://127.0.0.1:21000/query_gateway"),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut buff = Vec::new();
    rsp.encode(&mut buff).unwrap();

    rbase64::encode(&buff)
}

#[derive(Deserialize, Debug)]
pub struct QueryGatewayParameters {
    pub version: String,
    pub dispatch_seed: String,
}

#[instrument(skip(state))]
pub async fn query_gateway(
    State(state): State<Arc<RwLock<AppState>>>,
    parameters: Query<QueryGatewayParameters>,
) -> String {
    let mut lock = state.write().await;
    let config = lock
        .get_or_insert_hotfix(&parameters.version, &parameters.dispatch_seed)
        .await;

    let rsp = GateServer {
        ip: String::from("127.0.0.1"),
        port: 23301,
        asset_bundle_url: config.asset_bundle_url.clone(),
        asset_bundle_url_b: config.asset_bundle_url.clone(),
        ex_resource_url: config.ex_resource_url.clone(),
        lua_url: config.lua_url.clone(),
        ifix_version: String::from("0"),
        unk1: true,
        unk2: true,
        unk3: true,
        unk4: true,
        unk5: true,
        unk6: true,
        unk7: true,
        ..Default::default()
    };

    let mut buff = Vec::new();
    rsp.encode(&mut buff).unwrap();

    rbase64::encode(&buff)
}
