use std::collections::BTreeSet;

use salvo::prelude::*;

use crate::{model::*, utils};

#[handler]
pub async fn post(res: &mut Response) {
    let mut tokens = BTreeSet::new();
    if let Err(e) = utils::read_tokens_from_file(&mut tokens).await {
        return res.render(Json(Resp::<String> {
            code: Some(500),
            data: None,
            msg: Some(e.to_string()),
        }));
    }

    tokens.insert(utils::random_string(32));

    if let Err(e) = utils::set_auto_token(&mut tokens).await {
        return res.render(Json(Resp::<String> {
            code: Some(500),
            data: None,
            msg: Some(e.to_string()),
        }));
    }

    res.render(Json(Resp::<String> {
        code: Some(200),
        data: None,
        msg: None,
    }));
}

#[handler]
pub async fn get(res: &mut Response) {
    let tokens = utils::get_auth_token().await.unwrap();

    res.render(Json(Resp {
        code: Some(200),
        data: Some(tokens),
        msg: None,
    }));
}
