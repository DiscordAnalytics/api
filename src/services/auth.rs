use crate::app_env;

#[derive(Clone)]
pub struct AuthService;

impl AuthService {
    pub fn new() -> Self {
        Self
    }

    pub fn is_admin(&self, user_id: &str) -> bool {
        app_env!().admins.iter().any(|admin_id| admin_id == user_id)
    }
}
