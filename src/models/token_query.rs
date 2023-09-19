use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenQuery {
    token: String,
}

impl TokenQuery {
    pub fn get_token(&self) -> &str {
        &self.token
    }

    // pub fn get_query_string(&self) -> &str {
    //     format("token={}", &self.token)
    // }
}
