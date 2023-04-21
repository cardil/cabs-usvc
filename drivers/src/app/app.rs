use actix_web::{body, dev, get, middleware, App, Error, HttpRequest, HttpResponse};

use super::{config, events, index};

pub fn create() -> App<
    impl dev::ServiceFactory<
        dev::ServiceRequest,
        Response = dev::ServiceResponse<impl body::MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    App::new()
        .wrap(middleware::NormalizePath::trim())
        .wrap(middleware::Logger::default())
        .configure(config::configure)
        .service(index::endpoint)
        .service(events::endpoint)
        .service(health)
}

#[get("/health/{_:(ready|live)}")]
pub async fn health(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}
