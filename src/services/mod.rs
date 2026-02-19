use crate::repository::Repositories;

mod auth;

#[derive(Clone)]
pub struct Services {
    pub auth: auth::AuthService,
}

impl Services {
    pub fn new(repos: Repositories) -> Self {
        Self {
            auth: auth::AuthService::new(repos),
        }
    }
}
