#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;

use tide::utils::After;
use tide::{log, Server};

mod auth;
mod db;
mod interfaces;
mod middleware;
mod models;
mod setting;
mod state;
mod users;
mod utils;

use state::State;

lazy_static! {
    pub static ref CONFIG: setting::Setting =
        setting::Setting::new("Test").expect("Config Load Error");
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    dotenv::dotenv().ok();
    let state = State::new().await?;
    let mut app = Server::with_state(state.clone());
    app.with(After(utils::responser));
    app.at("/api/v1").nest({
        let mut api = Server::with_state(state.clone());
        auth::auth_router(&mut api);
        users::user_router(&mut api);
        interfaces::interface_router(&mut api);
        api
    });
    log::info!("app is running");
    app.listen(CONFIG.server.server.clone()).await?;
    Ok(())
}
