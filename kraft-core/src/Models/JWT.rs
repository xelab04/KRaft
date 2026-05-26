use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JWT {
    pub sub: String,
    pub roles: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}
