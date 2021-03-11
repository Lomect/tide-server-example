use tide::{log, Request};
use validator::Validate;

use crate::models::Interface;
use crate::utils::*;
use crate::{State, CONFIG};

pub(crate) async fn add_interface(mut req: Request<State>) -> tide::Result {
    let data: Interface = match req.body_json().await {
        Ok(data) => data,
        Err(error) => {
            return Responser::new(Some(format!("{}", error)), &status::BAD_REQUEST).to_result()
        }
    };
    if let Err(e) = data.validate() {
        return Responser::new(Some(e), &status::BAD_REQUEST).to_result();
    }
    log::error!("{:?}", data);
    Responser::new(Some(data), &status::OK).to_result()
}
