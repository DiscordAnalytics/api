mod health;

use apistos::web::{Scope, scope};

pub fn routes() -> Scope {
    scope("").service(health::routes())
}
