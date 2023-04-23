use actix_web::http::header;
use actix_web::{
    error,
    guard,
};
use futures::future::BoxFuture;
use std::future::Future;

use actix_web::guard::Guard;
use actix_web::{
    web,
    HttpRequest,
    HttpResponse,
    Result,
    Scope,
};

use super::entity::Driver;
use super::repository::{
    self,
    Repository,
};
use crate::app::config::{
    Db,
    State,
};
use crate::support::id::{
    Identifier,
    ID,
};
use crate::support::page::Page;

pub(crate) fn new(binding: Binding) -> Scope {
    web::scope("/drivers")
        .app_data(web::Data::new(binding))
        .service(
            web::resource("")
                .route(web::get().to(list))
                .route(web::post().guard(expects_json()).to(add))
                .route(web::put().guard(expects_json()).to(update)),
        )
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
    drv: web::Json<Driver>,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    log::debug!("drv: {:?}", drv);

    let db = state.db.clone();
    let mut repo = binding.repo_factory.call(db).await?;

    drv.validate().map_err(error::ErrorBadRequest)?;
    let id = ID {
        id:     Identifier::new(&state.clock),
        entity: drv.into_inner(),
    };

    repo.add(&id).await?;

    log::debug!("new id: {:?}", id.id);

    Ok(HttpResponse::Ok().json(&id))
}

async fn update(_req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

pub(crate) struct Binding {
    repo_factory: Box<dyn AsyncFactory>,
}

fn expects_json() -> impl Guard + Sized {
    guard::Header(header::CONTENT_TYPE.as_str(), "application/json")
}

impl Default for Binding {
    fn default() -> Self {
        Self {
            repo_factory: Box::new(repository::new),
        }
    }
}

// See: https://stackoverflow.com/a/66070319/844449
trait AsyncFactory {
    fn call(&self, args: Db)
        -> BoxFuture<'static, Result<Box<dyn Repository>>>;
}

impl<T, F> AsyncFactory for T
where
    T: Fn(Db) -> F,
    F: Future<Output = Result<Box<dyn Repository>>> + 'static + Send,
{
    fn call(
        &self,
        args: Db,
    ) -> BoxFuture<'static, Result<Box<dyn Repository>>> {
        Box::pin(self(args))
    }
}

#[cfg(test)]
mod tests {
    use crate::support::{
        id::ID,
        page::Pagination,
    };

    use super::*;
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
                .service(new(binding)),
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
