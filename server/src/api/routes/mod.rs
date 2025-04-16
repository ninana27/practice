use std::sync::Arc;
use warp::Filter;
use warp::Rejection;

use super::json_body;
use super::state::{with_state, AppState};

mod agents;
mod index;
mod jobs;

use agents::{get_agent_info, get_agents, post_agents};
use index::index;
use jobs::{create_job, get_agent_job, get_job_result, get_jobs, post_job_result};

pub fn routes(
    app_state: Arc<AppState>,
) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    let api = warp::path("api");
    let api_with_state = api.and(with_state(app_state));

    // GET api/
    let index = api.and(warp::path::end()).and(warp::get()).and_then(index);

    // GET api/jobs/
    let get_jobs = api_with_state
        .clone()
        .and(warp::path("jobs"))
        .and(warp::path::end())
        .and(warp::get())
        .and_then(get_jobs);

    // POST api/jobs
    let post_jobs = api_with_state
        .clone()
        .and(warp::path("jobs"))
        .and(warp::path::end())
        .and(warp::post())
        .and(json_body())
        .and_then(create_job);

    // GET api/jobs/{agent_id}
    let get_job = api_with_state
        .clone()
        .and(warp::path("jobs"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and_then(get_agent_job);

    // POST api/agents
    let post_agents = api_with_state
        .clone()
        .and(warp::path("agents"))
        .and(warp::path::end())
        .and(warp::post())
        .and(json_body())
        .and_then(post_agents);

    // GET api/jobs/result/{job_id}
    let get_job_result = api_with_state
        .clone()
        .and(warp::path("jobs"))
        .and(warp::path("result"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(get_job_result);

    // POST api/jobs/result
    let post_job_result = api_with_state
        .clone()
        .and(warp::path("jobs"))
        .and(warp::path("result"))
        .and(warp::path::end())
        .and(warp::post())
        .and(json_body())
        .and_then(post_job_result);

    // GET /api/agents
    let get_agents = api_with_state
        .clone()
        .and(warp::path("agents"))
        .and(warp::path::end())
        .and(warp::get())
        .and_then(get_agents);

    // GET /api/agents/{agent_id}
    let get_agent_info = api_with_state
        .clone()
        .and(warp::path("agents"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and_then(get_agent_info);

    // // GET /api/agents/{agent_id}/job
    // let get_agents_job = api_with_state
    //     .clone()
    //     .and(warp::path("agents"))
    //     .and(warp::path::param())
    //     .and(warp::path("job"))
    //     .and(warp::path::end())
    //     .and(warp::get())
    //     .and_then(get_agent_job);

    let routes = index
        .or(get_jobs)
        .or(post_jobs)
        .or(get_job)
        .or(post_agents)
        .or(get_agents)
        .or(get_job_result)
        .or(get_agent_info)
        .or(post_job_result);
    // .with(warp::log("server"))
    // .recover(super::handle_error);

    routes
}
