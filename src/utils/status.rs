pub(crate) type Res = (u32, String);

lazy_static! {
    pub(crate) static ref OK: Res = (1000, String::from("success"));
    pub(crate) static ref SYS_ERROR: Res = (1010, String::from("system error"));
    pub(crate) static ref BAD_REQUEST: Res = (1011, String::from("bad request"));
    pub(crate) static ref UNAUTH: Res = (1012, String::from("Unauthorized"));
    pub(crate) static ref TIME_OUT: Res = (1013, String::from("TIME_OUT"));
    pub(crate) static ref UNKNOWN: Res = (1020, String::from("UNKNOWN"));
}
