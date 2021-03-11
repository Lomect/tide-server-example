use validator::Validate;

#[derive(Deserialize, Validate)]
pub(crate) struct Login {
    #[validate(email(message = "Please input correct email"))]
    pub(crate) email: String,
    #[validate(length(min = 3, message = "password length too min"))]
    pub(crate) password: String,
}

#[derive(Deserialize, Validate)]
pub(crate) struct Register {
    #[validate(length(min = 4, message = "username min length 4"))]
    pub(crate) username: String,
    #[validate(email(message = "email type error"))]
    pub(crate) email: String,
    pub(crate) phone: String,
    #[validate(
        must_match = "confirm",
        length(min = 6, message = "password length min 6")
    )]
    pub(crate) password: String,
    pub(crate) confirm: String,
}

#[derive(Deserialize, Validate)]
pub(crate) struct Resend {
    #[validate(email(message = "email type error"))]
    pub(crate) email: String,
    pub(crate) forget: Option<bool>,
}

#[derive(Deserialize, Validate)]
pub(crate) struct ResetPwd {
    #[validate(
        must_match = "confirm",
        length(min = 6, message = "password length min 6")
    )]
    pub(crate) password: String,
    pub(crate) confirm: String,
}
