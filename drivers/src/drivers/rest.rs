use actix_web::dev::HttpServiceFactory;
use actix_web::guard::Guard;
use actix_web::http::header;
use actix_web::{
    error,
    guard,
};
use actix_web::{
    web,
    HttpRequest,
    HttpResponse,
    Result,
};

use crate::app::config::State;
use crate::drivers::entity::{
    NewDriver,
    Type,
};
use crate::drivers::Binding;
use crate::support::id::{
    Identifier,
    ID,
};
use crate::support::page::Page;

use super::entity::Driver;

pub(crate) fn new() -> impl HttpServiceFactory + 'static {
    web::scope("/drivers")
        .service(
            web::resource("")
                .route(web::get().to(list))
                .route(web::post().guard(expects_json()).to(add)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to(get))
                .route(web::put().guard(expects_json()).to(update)),
        )
        .service(web::resource("/{id}/activate").route(web::put().to(activate)))
        .service(
            web::resource("/{id}/deactivate").route(web::put().to(deactivate)),
        )
        .service(web::resource("/{id}/graduate").route(web::put().to(graduate)))
}

async fn get(
    path: web::Path<i64>,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    let id = Identifier::from(path.into_inner());
    log::debug!("id: {:?}", id);

    let db = state.db.clone();
    let mut repo = binding.repo_factory.call(db).await?;

    let drv = repo.get(&id).await?;

    Ok(HttpResponse::Ok().json(&drv))
}

async fn list(
    req: HttpRequest,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    let db = state.db.clone();
    let mut repo = binding.repo_factory.call(db).await?;

    let page = Page::try_from(&req)?;
    log::debug!("page: {:?}", page);
    let total = repo.count().await?;
    let pagin = page.to_pagination(total);
    log::debug!("Pagination: {:?}", pagin);

    let list = repo.list(&pagin.page).await?;

    let mut res = HttpResponse::Ok().json(&list);
    pagin.onto_response(&mut res)?;

    Ok(res)
}

async fn add(
    drv: web::Json<NewDriver>,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    log::debug!("drv: {:?}", drv);

    let db = state.db.clone();
    let mut repo = binding.repo_factory.call(db).await?;

    drv.validate(&state.clock).map_err(error::ErrorBadRequest)?;
    let id = ID {
        id:     Identifier::new(&state.clock),
        entity: drv.into_inner().onto(&Driver::default()),
    };

    repo.set(&id).await?;

    log::debug!("new id: {:?}", id.id);

    Ok(HttpResponse::Ok().json(&id))
}

async fn update(
    path: web::Path<i64>,
    drv: web::Json<NewDriver>,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    let id = Identifier::from(path.into_inner());
    log::debug!("id: {:?}", id);

    log::debug!("drv: {:?}", drv);
    drv.validate(&state.clock).map_err(error::ErrorBadRequest)?;

    let db = state.db.clone();
    let mut repo = binding.repo_factory.call(db).await?;

    let curr = repo.get(&id).await?;
    let upd = drv.into_inner().onto(&curr);
    let inst = ID {
        id,
        entity: upd.clone(),
    };
    log::debug!("to update: {:?}", &inst);
    repo.set(&inst).await?;

    Ok(HttpResponse::Ok().json(&upd))
}

async fn activate(
    path: web::Path<i64>,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    let id = Identifier::from(path.into_inner());
    log::debug!("id: {:?}", id);

    let db = state.db.clone();
    let mut repo = binding.repo_factory.call(db).await?;

    let curr = repo.get(&id).await?;
    let upd = curr
        .activate(&state.clock)
        .map_err(error::ErrorBadRequest)?;
    let inst = ID {
        id,
        entity: upd.clone(),
    };
    log::debug!("to update: {:?}", &inst);
    repo.set(&inst).await?;

    Ok(HttpResponse::Ok().json(&upd))
}

async fn deactivate(
    path: web::Path<i64>,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    let id = Identifier::from(path.into_inner());
    log::debug!("id: {:?}", id);

    let db = state.db.clone();
    let mut repo = binding.repo_factory.call(db).await?;

    let curr = repo.get(&id).await?;
    let upd = curr.deactivate();
    let inst = ID {
        id,
        entity: upd.clone(),
    };
    log::debug!("to update: {:?}", &inst);
    repo.set(&inst).await?;

    Ok(HttpResponse::Ok().json(&upd))
}

async fn graduate(
    path: web::Path<i64>,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    let id = Identifier::from(path.into_inner());
    log::debug!("id: {:?}", id);

    let db = state.db.clone();
    let mut repo = binding.repo_factory.call(db).await?;

    let curr = repo.get(&id).await?;
    let upd = curr.with_type(Type::Regular);
    let inst = ID {
        id,
        entity: upd.clone(),
    };
    log::debug!("to update: {:?}", &inst);
    repo.set(&inst).await?;

    Ok(HttpResponse::Ok().json(&upd))
}

fn expects_json() -> impl Guard + Sized {
    guard::Header(header::CONTENT_TYPE.as_str(), "application/json")
}

#[cfg(test)]
mod tests {
    use actix_web::{
        body::to_bytes,
        http::{
            header,
            StatusCode,
        },
        test::{
            self,
            TestRequest,
        },
        web::{
            Bytes,
            Data,
        },
        App,
        Result,
    };

    use crate::support::{
        id::ID,
        page::Pagination,
    };

    use super::*;

    trait BodyTest {
        fn as_str(&self) -> &str;
    }

    impl BodyTest for Bytes {
        fn as_str(&self) -> &str {
            std::str::from_utf8(self).unwrap()
        }
    }

    #[test_log::test(actix_web::test)]
    async fn test_drivers_get() -> Result<()> {
        let state = State::default();
        let binding = Binding::default();
        let req = TestRequest::get()
            .uri("/drivers")
            .append_header((header::RANGE, "page=2-40"))
            .to_http_request();
        let res =
            list(req.clone(), Data::new(state), Data::new(binding)).await?;

        assert_list_response(res).await
    }

    #[test_log::test(actix_web::test)]
    async fn e2e_test_drivers_get() -> Result<()> {
        let state = State::default();
        let binding = Binding::default();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(state.config.clone()))
                .app_data(Data::new(state))
                .app_data(Data::new(binding))
                .service(new()),
        )
        .await;

        let req = TestRequest::get()
            .uri("/drivers")
            .append_header((header::RANGE, "page=2-40"))
            .to_request();
        let res: HttpResponse = test::call_service(&app, req).await.into();

        assert_list_response(res).await
    }

    async fn assert_list_response(res: HttpResponse) -> Result<()> {
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get(header::ACCEPT_RANGES),
            Some(&header::HeaderValue::from_static("page"))
        );
        let pagin = Pagination::try_from(&res)?;
        assert_eq!(pagin.page.per, 40);
        assert!(pagin.page.num >= 1);
        assert!(pagin.total >= 0);

        let body = to_bytes(res.into_body()).await?;
        let drvs: Vec<ID<Driver>> = serde_json::from_slice(&body)?;
        assert!(drvs.len() <= 40);
        drvs.iter().for_each(|drv| {
            assert_ne!(drv.id.int(), 0);
            assert_ne!(drv.entity.name, "");
            assert_ne!(drv.entity.surname, "");
        });

        Ok(())
    }
}
