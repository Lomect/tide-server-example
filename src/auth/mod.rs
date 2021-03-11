mod routers;
mod schema;

use tide::Server;

use crate::State;
pub(crate) use schema::Register;

pub fn auth_router(app: &mut Server<State>) {
    let mut auth = app.at("/auth");
    auth.at("/login").post(routers::login);
    auth.at("/register").post(routers::register);
    auth.at("/resend").post(routers::resend);
    auth.at("/confirm").post(routers::confirm);
    auth.at("/resetpwd").post(routers::reset_pwd);
}
