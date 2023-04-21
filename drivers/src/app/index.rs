use actix_web::{web, HttpRequest, get};
use log::info;

use crate::app::config::Config;

#[get("/")]
pub async fn endpoint(req: HttpRequest, config: web::Data<Config>) -> String {
    info!("{:#?}", req);
    format!("Hello {}!\n", config.name)
}

#[cfg(test)]
mod tests {
    use crate::app::config;

    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn get() {
        let app = test::init_service(
            App::new()
                .configure(config::configure)
                .service(endpoint)
        ).await;
        let req = test::TestRequest::get()
            .uri("/")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert_eq!("Hello world!\n", body);
    }

}
