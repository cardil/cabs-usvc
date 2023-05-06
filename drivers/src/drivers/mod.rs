use crate::{
    app::config::Db,
    drivers::repository::Repository,
};
use actix_web::{
    dev::HttpServiceFactory,
    Result,
};
use futures::future::BoxFuture;
use std::future::Future;

pub mod entity;
mod repository;
pub mod rest;
pub mod service;

pub fn routes() -> impl HttpServiceFactory + 'static {
    rest::new()
}

pub(crate) struct Binding {
    pub(crate) repo_factory: Box<dyn AsyncFactory>,
}

impl Default for Binding {
    fn default() -> Self {
        Self {
            repo_factory: Box::new(repository::new),
        }
    }
}

// See: https://stackoverflow.com/a/66070319/844449
pub(crate) trait AsyncFactory {
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
