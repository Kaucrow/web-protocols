pub const SSS_COOKIE_NAME: &'static str = "session";

pub struct SessionPublicToken {
    pub uuid_key: &'static str,
}

pub struct SessionDataToken {
    pub session_key: &'static str,
}

pub const SSS_PUB_TK: SessionPublicToken = SessionPublicToken {
    uuid_key: "session_uuid",
};

pub const SSS_DATA_TK: SessionDataToken = SessionDataToken {
    session_key: "session",
};