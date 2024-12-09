use crate::prelude::*;

#[derive(Deserialize)]
pub struct NewUser {
    pub email: String,
    pub name: String,
    pub password: String,
}