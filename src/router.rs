use salvo::prelude::*;

use crate::handler;

pub fn router() -> Router {
    Router::with_path("v1")
        .push(Router::with_path("info").get(handler::info))
        .push(Router::with_path("repair").get(handler::repair))
}
