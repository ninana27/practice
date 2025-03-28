use std::{sync::Arc, time::Duration};
use uuid::Uuid;
use warp::{http::StatusCode, Rejection};

use crate::api::state::AppState;
use crate::share;

pub async fn get_jobs(state: Arc<AppState>) -> Result<impl warp::Reply, Rejection> {
    let jobs = state.service.list_jobs().await?;

    let res = share::Response::ok(jobs);
    let res_json = warp::reply::json(&res);
    Ok(warp::reply::with_status(res_json, StatusCode::OK))
}

pub async fn create_job(
    state: Arc<AppState>,
    input: share::CreateJob,
) -> Result<impl warp::Reply, warp::Rejection> {
    let job = state.service.create_job(input).await?;

    let res = share::Response::ok(job);
    let res_json = warp::reply::json(&res);
    Ok(warp::reply::with_status(res_json, StatusCode::OK))
}

pub async fn get_agent_job(
    state: Arc<AppState>,
    agent_id: Uuid,
) -> Result<impl warp::Reply, warp::Rejection> {
    let sleep_for = Duration::from_secs(1);

    // long polling: 5 secs
    for _ in 0..5u64 {
        match state.service.get_agent_job(agent_id).await? {
            Some(job) => {
                let agent_job = share::AgentJob {
                    id: job.id,
                    command: job.command,
                    args: job.args.0,
                };

                let res = share::Response::ok(agent_job);
                let res_json = warp::reply::json(&res);
                return Ok(warp::reply::with_status(res_json, StatusCode::OK));
            }
            None => tokio::time::sleep(sleep_for).await,
        }
    }

    // if no job is found, return empty response
    let res = share::Response::<Option<()>>::ok(None);
    let res_json = warp::reply::json(&res);
    Ok(warp::reply::with_status(res_json, StatusCode::OK))
}

pub async fn get_job_result(
    state: Arc<AppState>,
    job_id: Uuid,
) -> Result<impl warp::Reply, Rejection> {
    let job = state.service.list_job_result(job_id).await?;

    let res = share::Response::ok(job);
    let res_json = warp::reply::json(&res);
    Ok(warp::reply::with_status(res_json, StatusCode::OK))
}

pub async fn post_job_result(
    state: Arc<AppState>,
    job_result: share::UpdateJobResult,
) -> Result<impl warp::Reply, warp::Rejection> {
    state.service.update_job_result(job_result).await?;

    let res = share::Response::ok(true);
    let res_json = warp::reply::json(&res);
    Ok(warp::reply::with_status(res_json, StatusCode::OK))
}
