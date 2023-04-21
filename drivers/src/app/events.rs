use actix_web::{web, HttpRequest, post};

use crate::app::config::Config;

#[post("/")]
pub async fn endpoint(_req: HttpRequest, config: web::Data<Config>) -> String {
    format!("Thanks {}!\n", config.name)
}

#[cfg(test)]
mod tests {
    use crate::app::config;

    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn post() {
        let app = test::init_service(
            App::new()
                .configure(config::configure)
                .service(endpoint)
        ).await;
        let req = test::TestRequest::post()
            .uri("/")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert_eq!("Thanks world!\n", body);
    }
}