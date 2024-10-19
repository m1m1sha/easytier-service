use salvo::prelude::*;

use crate::{constant, easytier, model::*};

#[handler]
pub async fn info(res: &mut Response) {
    let mut easytier_list = vec![];

    if easytier::exists() {
        // 获取 easytier 列表, 暂时只支持单独一个实例
        if let Ok(version) = easytier::version().await {
            easytier_list.push(REasytier {
                instance_id: None,
                instance_name: None,
                running: false,
                version,
            });
        }
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

#[handler]
pub async fn repair(req: &mut Request, res: &mut Response) {
    let replace = req.query::<bool>("replace").unwrap_or_default();
    match easytier::check_exists(replace).await {
        Ok(_) => {
            tracing::info!("repair easytier success");

            match easytier::version().await {
                Ok(version) => res.render(Json(Resp {
                    code: Some(200),
                    data: Some(RRepair { version }),
                    msg: None,
                })),
                Err(e) => res.render(Json(Resp::<EVersion> {
                    code: Some(500),
                    data: None,
                    msg: Some(e.to_string()),
                })),
            }
        }
        Err(e) => {
            tracing::error!("repair easytier failed: {}", e);
            res.render(Json(Resp::<EVersion> {
                code: Some(500),
                data: None,
                msg: Some(e.to_string()),
            }));
        }
    };
}
