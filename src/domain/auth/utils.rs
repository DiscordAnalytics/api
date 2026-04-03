use crate::app_env;

pub fn is_admin(user_id: &str) -> bool {
    app_env!().admins.iter().any(|admin_id| admin_id == user_id)
}
