use actix_web::web;

/// Run custom configuration as part of the application building
/// process.
///
/// This function should contain all custom configuration for your function application.
///
/// ```rust
/// fn configure(cfg: &mut web::ServiceConfig) {
///     let db_driver = my_db();
///     cfg.app_data(Data::new((db_driver.clone()));
/// }
/// ```
///
/// Then you can use configured resources in your function.
///
/// ```rust
/// pub async fn index(
///     req: HttpRequest,
///     driver: web::Data<DbDriver>,
/// ) -> HttpResponse {
///     HttpResponse::NoContent()
/// }
pub fn configure(cfg: &mut web::ServiceConfig) {
    log::info!("Configuring service");
    cfg.app_data(web::Data::new(
        Config::default()
    ));
}

pub fn setup_logger() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
}

pub fn get_port() -> u16 {
    match std::env::var("PORT") {
        Ok(v) => v.parse().unwrap(),
        Err(_) => 8080,
    }
}

/// An example of the function configuration structure.
#[derive(Clone)]
pub struct Config {
    pub name: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            name: String::from("world"),
        }
    }
}
