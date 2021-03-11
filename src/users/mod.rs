mod routers;
mod schema;

use tide::Server;

use crate::State;
use crate::middleware::LoginMiddleware;

pub fn user_router(api: &mut Server<State>) {
    let mut user = api.with(LoginMiddleware).at("/user");
    user.at("/").get(routers::get_user);
}

