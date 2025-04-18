use warp::{http::StatusCode, Rejection};

pub async fn index() -> Result<impl warp::Reply, Rejection> {
    let welc = "Welcome to api!";
    Ok(warp::reply::with_status(welc, StatusCode::OK))
}
