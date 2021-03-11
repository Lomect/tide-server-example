mod routers;
mod schema;

use tide::Server;

use crate::State;
use routers::add_interface;
use crate::middleware::LoginMiddleware;

pub(crate) fn interface_router(app: &mut Server<State>) {
    let mut interface = app.with(LoginMiddleware).at("/interface");
    interface.at("/add").post(add_interface);
}
