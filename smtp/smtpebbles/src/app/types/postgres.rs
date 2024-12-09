use crate::prelude::*;
use anyhow::Result;

pub struct User {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl TryFrom<PgRow> for User {
    type Error = anyhow::Error;
    fn try_from(row: PgRow) -> Result<Self> {
        Ok(Self {
            email: row.try_get("email")?,
            name: row.try_get("name")?,
            password: row.try_get("password")?,
        })
    }
}