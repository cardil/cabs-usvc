use crate::support::clock::Clock;
use std::env;

pub fn get_port() -> u16 {
    match env::var("PORT") {
        Ok(v) => v.parse().unwrap(),
        Err(_) => 8081,
    }
}

pub fn setup_logger(env: &Environment) {
    let level = match env {
        Environment::Development => Some(log::LevelFilter::Debug),
        Environment::Production => Some(log::LevelFilter::Info),
    };
    let level = match env::var(env_logger::DEFAULT_FILTER_ENV) {
        Ok(l) => match l.is_empty() {
            true => level,
            false => None,
        },
        Err(_) => level,
    };
    let mut b = env_logger::builder();
    if level.is_some() {
        b.filter_level(level.unwrap());
    }
    match b.try_init() {
        Ok(_) => (),
        Err(err) => eprintln!("Error initializing logger: {}", err),
    };
}

#[derive(Clone, Debug)]
pub struct Config {
    pub db:          DbConfig,
    pub environment: Environment,
    pub knative:     Knative,
    pub name:        String,
}

#[derive(Clone, Debug)]
pub struct Knative {
    pub sink: String,
}

#[derive(Clone, Debug)]
pub enum Environment {
    Development,
    Production,
}

#[derive(Clone, Debug)]
pub struct DbConfig {
    pub uri:  String,
    pub user: Option<String>,
    pub pass: Option<String>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub config: Config,
    pub db:     Db,
    pub clock:  Clock,
}

#[derive(Clone, Debug)]
pub struct Db {
    pub client: Option<redis::Client>,
}

impl Default for Config {
    fn default() -> Config {
        let db = DbConfig {
            uri:  env_or("APP_DB_URI", "redis://127.0.0.1/"),
            user: env::var("APP_DB_USER").ok(),
            pass: env::var("APP_DB_PASS").ok(),
        };

        let def_env = match cfg!(debug_assertions) {
            true => Environment::Development,
            false => Environment::Production,
        };
        let environment = match env::var("APP_ENV")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "dev" => Environment::Development,
            "development" => Environment::Development,
            "prod" => Environment::Production,
            "production" => Environment::Production,
            _ => def_env,
        };

        let name = String::from("world");

        let knative = Knative {
            sink: env_or("K_SINK", "http://localhost:31111/"),
        };

        Config {
            db,
            environment,
            knative,
            name,
        }
    }
}

impl Default for State {
    fn default() -> State {
        let config = Config::default();
        let db = Db {
            client: redis::Client::open(config.db.uri.as_str()).ok(),
        };
        let clock = Clock::default();
        State { config, db, clock }
    }
}

fn env_or(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(v) => v,
        Err(_) => default.to_string(),
    }
}
