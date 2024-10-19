use crate::handlers;
use salvo::prelude::*;

pub fn router() -> Router {
    Router::with_path("v1")
        .push(Router::with_path("info").get(handlers::info))
        .push(Router::with_path("repair").post(handlers::repair))
        .push(
            Router::with_path("authToken")
                .get(handlers::auth_token::get)
                .post(handlers::auth_token::post),
        )
}
