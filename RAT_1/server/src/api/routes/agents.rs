use std::sync::Arc;
use uuid::Uuid;
use warp::{http::StatusCode, Rejection};

use crate::share;

use crate::api::state::AppState;

pub async fn post_agents(
    state: Arc<AppState>,
    register: share::AgentRegister,
) -> Result<impl warp::Reply, Rejection> {
    let anget_info = state.service.register_agent(register).await?;

    let res = share::Response::ok(anget_info);
    let res_json = warp::reply::json(&res);
    Ok(warp::reply::with_status(res_json, StatusCode::OK))
}

pub async fn get_agents(state: Arc<AppState>) -> Result<impl warp::Reply, Rejection> {
    let angets_info = state.service.list_agents().await?;

    let res = share::Response::ok(angets_info);
    let res_json = warp::reply::json(&res);
    Ok(warp::reply::with_status(res_json, StatusCode::OK))
}

pub async fn get_agent_info(
    state: Arc<AppState>,
    agent_id: Uuid,
) -> Result<impl warp::Reply, Rejection> {
    let anget_info = state.service.get_agent(agent_id).await?;

    let res = share::Response::ok(anget_info);
    let res_json = warp::reply::json(&res);
    Ok(warp::reply::with_status(res_json, StatusCode::OK))
}
