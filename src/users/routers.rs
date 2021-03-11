use super::schema::{GetUser, ResUser};
use crate::{
    db::Options,
    models::USER,
    utils::{status, Responser},
    State,
};
use mongodb::bson::{doc, from_document};
use tide::log;

pub async fn get_user(mut req: tide::Request<State>) -> tide::Result {
    log::error!("get user");
    let filter: GetUser = match req.query() {
        Ok(res) => res,
        Err(e) => return Responser::new(Some(e.to_string()), &status::BAD_REQUEST).to_result(),
    };
    log::error!("filter {:?}", filter);

    let skip = (filter.page.page_num - 1) * filter.page.page_size;
    let fileds = Some(doc! {"username": 1, "email": 1, "phone": 1, "create_at": 1});
    let mut opt = Options::new(
        &USER,
        None,
        Some(filter.page.page_size as i64),
        Some(skip as i64),
        None,
        fileds,
    );
    match filter.name {
        Some(name) => opt.filter = Some(doc! {"username": name}),
        None => {}
    }
    let mongo = &req.state().mongo;
    let users = match mongo.find(opt).await {
        Ok(res) => res,
        Err(e) => return Responser::new(Some(e.to_string()), &status::BAD_REQUEST).to_result(),
    };
    let data: Vec<Option<ResUser>> = users
        .into_iter()
        .map(|s| from_document::<ResUser>(s).ok())
        .filter(|s| s.is_some())
        .collect();
    Responser::new(Some(data), &status::OK).to_result()
}
