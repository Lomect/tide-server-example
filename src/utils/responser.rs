use super::status;
use tide::{log, prelude::*, Response, StatusCode};

#[derive(Serialize)]
pub struct Responser<T: Serialize> {
    pub code: u32,
    pub data: Option<T>,
    pub msg: String,
}

#[allow(unused)]
impl<T: Serialize> Responser<T> {
    pub fn new(t: Option<T>, res: &status::Res) -> Self {
        Self {
            code: res.0,
            msg: res.1.clone(),
            data: t,
        }
    }

    pub fn to_result(&self) -> tide::Result {
        let s_json = json!(self);
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(s_json);
        Ok(res)
    }
}

pub async fn responser(mut res: Response) -> tide::Result {
    if res.take_error().is_some() {
        let msg = match res.take_error() {
            Some(msg) => format!("{}", msg),
            None => String::from("UNKNOWN错误，请查看日志"),
        };
        log::error!("{:?}", res.status());
        match res.status() {
            StatusCode::Ok => Ok(res),
            StatusCode::InternalServerError => {
                Responser::new(Some(msg), &status::SYS_ERROR).to_result()
            }
            StatusCode::UnprocessableEntity => {
                Responser::new(Some("请求参数有误"), &status::SYS_ERROR).to_result()
            }
            StatusCode::Unauthorized => Responser::new(Some(msg), &status::UNAUTH).to_result(),
            StatusCode::BadRequest => Responser::new(Some(msg), &status::BAD_REQUEST).to_result(),
            _ => Responser::new(Some("UNKNOWN"), &status::UNKNOWN).to_result(),
        }
    } else {
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::status;
    use super::Responser;
    use tide::prelude::*;
    #[test]
    fn test_code_message_to_response() {
        #[derive(Serialize)]
        struct A {
            a: String,
        }

        let a = A { a: "A".to_string() };
        let res = Responser::new(Some(a), &status::BAD_REQUEST);
        assert_eq!(res.code, status::BAD_REQUEST.0);
    }
}
