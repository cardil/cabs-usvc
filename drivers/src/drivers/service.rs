use crate::{
    app::config::{
        Config,
        State,
    },
    drivers::{
        repository::Repository,
        Binding,
    },
    support::{
        cloudevents::Sender,
        id::{
            Identifier,
            Subject,
        },
        money::Money,
    },
};
use actix_web::{
    error,
    web,
    Result,
};
use cloudevents::{
    AttributesReader,
    Data,
    Event,
    EventBuilder,
    EventBuilderV10,
};
use serde::{
    Deserialize,
    Serialize,
};

pub struct Service {
    config: Config,
    repo:   Box<dyn Repository>,
}

impl Service {
    pub async fn calculate_fee(&mut self, ce: Event) -> Result<()> {
        let calc_fee_intent = Self::unwrap_calculatefee(ce)?;
        let subject = calc_fee_intent.id.clone();

        log::debug!("calculate fee for: {:?}", calc_fee_intent);
        let drv = self.repo.get(&calc_fee_intent.entity.driver_id).await?;

        let fee = drv.calculate_fee(&calc_fee_intent.entity.transit_price);

        log::debug!("fee value: {:?}", fee);

        let driverfee_event = DriverFeeEvent {
            driver_id: calc_fee_intent.entity.driver_id,
            fee,
        };

        let mut builder = driverfee_event.to_builder();
        if let Some(id) = subject {
            builder = builder.subject(id);
        }
        let ce = builder.build().map_err(error::ErrorInternalServerError)?;

        Sender::new(&self.config).send(ce).await?;

        Ok(())
    }

    fn unwrap_calculatefee(ce: Event) -> Result<Subject<CalculateFeeEvent>> {
        let ct = ce.datacontenttype();
        if ct != Some("application/json") {
            return Err(error::ErrorBadRequest(format!(
                "unsupported content type: {:#?}",
                ct
            )));
        }

        let data = match ce.data() {
            Some(data) => serde_json::Value::try_from(data.clone())?,
            None => return Err(error::ErrorBadRequest("missing data")),
        };

        let entity: CalculateFeeEvent =
            match serde_json::from_value(data.clone()) {
                Ok(event) => event,
                Err(err) => {
                    return Err(error::ErrorBadRequest(format!(
                        "failed to parse event: {}",
                        err
                    )))
                }
            };

        let id = ce.subject().map(|s| s.to_string());

        Ok(Subject { id, entity })
    }
}

pub(crate) async fn new(
    state: web::Data<State>,
    binding: web::Data<Binding>,
) -> Result<Service> {
    let repo = binding.repo_factory.call(state.db.clone()).await?;
    let config = state.config.clone();
    Ok(Service { repo, config })
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct CalculateFeeEvent {
    driver_id:     Identifier,
    transit_price: Money,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct DriverFeeEvent {
    driver_id: Identifier,
    fee:       Money,
}

impl Into<Data> for &DriverFeeEvent {
    fn into(self) -> Data {
        Data::Json(serde_json::to_value(self).unwrap())
    }
}

impl DriverFeeEvent {
    fn to_builder(&self) -> EventBuilderV10 {
        EventBuilderV10::default()
            .source("usvc://cabs/drivers")
            .ty("cabs.drivers.driver-fee")
            .data("application/json", self)
    }
}
