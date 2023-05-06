use actix_web::{
    dev::HttpServiceFactory,
    error,
    web,
    HttpResponse,
    Result,
};

use crate::app::config::State;
use crate::drivers::{
    service,
    Binding,
};
use cloudevents::{
    AttributesReader,
    Event,
};

pub fn routes() -> impl HttpServiceFactory + 'static {
    web::resource("/").route(web::post().to(recv))
}

async fn recv(
    ce: Event,
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<HttpResponse> {
    log::info!("Received event:\n{}", ce);

    let mut svc = service::new(state, binding).await?;

    match ce.ty() {
        "cabs.drivers.calculate-fee" => svc.calculate_fee(ce).await,
        _ => Err(error::ErrorBadRequest("unsupported event type")),
    }?;

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        test,
        App,
    };

    #[actix_web::test]
    async fn post() {
        let app = test::init_service(App::new().service(routes())).await;
        let req = test::TestRequest::post().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert_eq!("Thanks world!\n", body);
    }
}
