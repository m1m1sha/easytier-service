use salvo::prelude::*;

use crate::{constant, easytier, model::*};

#[handler]
pub async fn info(res: &mut Response) {
    let mut easytier_list = vec![];

    if easytier::exists() {
        // 获取 easytier 列表, 暂时只支持单独一个实例
        easytier_list.push(REasytier {
            instance_id: None,
            instance_name: None,
            running: false,
            version: easytier::version(),
        });
    };

    res.render(Json(Resp {
        code: Some(200),
        data: Some(RInfo {
            version: constant::VERSION.to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            list: easytier_list,
        }),
        msg: None,
    }));
}
