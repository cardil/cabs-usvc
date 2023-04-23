use actix_web::Scope;

pub mod entity;
pub mod endpoint;
mod repository;

pub fn scope() -> Scope {
    endpoint::new(endpoint::Binding::default())
}
