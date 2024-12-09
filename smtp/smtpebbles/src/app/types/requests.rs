use crate::prelude::*;

#[derive(Deserialize)]
pub struct NewUser {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}