use std::{convert::Infallible, sync::atomic::Ordering, time::Duration};

use warp::Filter;

use crate::state::AppState;

#[derive(Debug)]
pub struct HealthcheckBuilder {
    state: Option<AppState>,
}

impl HealthcheckBuilder {
    pub fn state(mut self, state: AppState) -> Self {
        self.state = Some(state);
        self
    }

    pub async fn build(self) -> Result<Healthcheck, Infallible> {
        let state = self.state.expect("state must be set");
        Ok(Healthcheck { state })
    }
}

#[derive(Debug)]
pub struct Healthcheck {
    state: AppState,
}

impl Healthcheck {
    pub fn builder() -> HealthcheckBuilder {
        HealthcheckBuilder { state: None }
    }

    pub async fn run(&mut self) {
        // create a simple warp webserver on port 3000
        // that returns a 200 if the state is healthy
        // and a 500 if the state is unhealthy

        let start_time = self.state.start_time;
        let num_connected = self.state.num_connected.clone();

        let healthcheck = warp::path!("healthcheck").and(warp::get()).map(move || {
            // return if uptime less than 1 minute
            if start_time.elapsed() < Duration::from_secs(60) {
                return warp::reply::with_status("OK", warp::http::StatusCode::OK);
            }

            if num_connected.load(Ordering::Relaxed) < 1 {
                return warp::reply::with_status(
                    "NOT OK - NOT ENOUGH SERVERS",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                );
            }

            warp::reply::with_status("OK", warp::http::StatusCode::OK)
        });

        let server = warp::serve(healthcheck);

        server.bind(([0, 0, 0, 0], 3000)).await;
    }
}
