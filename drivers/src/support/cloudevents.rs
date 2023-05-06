use crate::app::config::Config;
use actix_web::{
    error,
    Result,
};
use cloudevents::{
    binding::reqwest::RequestBuilderExt,
    AttributesReader,
    Event,
};

pub struct Sender {
    client: reqwest::Client,
    sink:   String,
}

impl Sender {
    pub fn new(cfg: &Config) -> Self {
        let client = reqwest::Client::new();
        let url = cfg.knative.sink.clone();
        Self { client, sink: url }
    }

    pub async fn send(&self, ce: Event) -> Result<()> {
        log::debug!("sending {} event to {}:\n{:?}", ce.ty(), &self.sink, ce,);

        let response = self
            .client
            .post(&self.sink)
            .event(ce)
            .map_err(error::ErrorInternalServerError)?
            .send()
            .await
            .map_err(error::ErrorInternalServerError)?;

        match response.status().is_success() {
            true => Ok(()),
            false => {
                log::error!("failed to send event: {:#?}", response);
                Err(error::ErrorInternalServerError(format!(
                    "failed to send event: {}",
                    response.status()
                )))
            }
        }
    }
}
