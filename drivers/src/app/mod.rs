pub mod config;
pub mod events;
pub mod index;

use actix_web::{
    body,
    dev,
    get,
    middleware,
    web::Data,
    App,
    Error,
    HttpRequest,
    HttpResponse,
};

use crate::drivers;
use crate::drivers::Binding;

pub fn create(
    state: config::State,
) -> App<
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
        .app_data(Data::new(state.config.clone()))
        .app_data(Data::new(state))
        .app_data(Data::new(Binding::default()))
        .service(index::endpoint)
        .service(events::routes())
        .service(health)
        .service(drivers::routes())
}

#[get("/health/{_:(ready|live)}")]
async fn health(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}
