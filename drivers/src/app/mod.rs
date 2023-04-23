pub mod config;
pub mod events;
pub mod index;

use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::HttpRequest;
use actix_web::Error;
use actix_web::App;
use actix_web::middleware;
use actix_web::get;
use actix_web::dev;
use actix_web::body;

use crate::drivers;

pub fn create(state: config::State) -> App<
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
        .service(index::endpoint)
        .service(events::endpoint)
        .service(health)
        .service(drivers::scope())
}

#[get("/health/{_:(ready|live)}")]
async fn health(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}
