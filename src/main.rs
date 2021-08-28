extern crate envconfig;
extern crate log;
extern crate redis;
extern crate reqwest;
extern crate serde_json;
extern crate simple_logger;

use std::collections::HashMap;
use std::process::{exit, id};
use std::time::Duration;

use async_std::channel::{bounded, Receiver, Sender};
use async_std::task;
use envconfig::Envconfig;
use futures::join;
use log::{debug, error, info, LevelFilter, warn};
use redis::{Commands, Value};
use redis::streams::{StreamId, StreamKey, StreamReadOptions, StreamReadReply};
use reqwest::{Client, StatusCode};
use serde_json::json;
use simple_logger::SimpleLogger;

#[derive(Envconfig)]
#[derive(Debug)]
#[derive(Clone)]
struct Config {
    #[envconfig(from = "REDIS_URL", default = "redis://redis:6379/0")]
    pub redis_url: String,

    #[envconfig(from = "STREAM_KEY", default = "/auth/login")]
    pub stream_key: String,

    #[envconfig(from = "STREAM_GROUP", default = "email-verification")]
    pub stream_group: String,

    #[envconfig(from = "VERIFICATION_URL", default = "https://2read.online/auth/verificate")]
    pub verification_url: String,

    #[envconfig(from = "MAILGUN_DOMAIN", default = "2read.online")]
    pub mailgun_domain: String,

    #[envconfig(from = "MAILGUN_API_KEY")]
    pub mailgun_api_key: String,

    #[envconfig(from = "MAILGUN_FROM")]
    pub mailgun_from: String,

    #[envconfig(from = "MAILGUN_SUBJECT", default = "EMail Verification")]
    pub mailgun_subject: String,

    #[envconfig(from = "MAILGUN_TEMPLATE")]
    pub mailgun_template: String,
}

#[derive(Debug)]
struct VerificationMessage {
    pub email: String,
    pub hash: String,
}

fn parse_field(map: &HashMap<String, Value>, key: &'static str) -> Option<String> {
    match map.get(key) {
        Some(Value::Data(bytes)) => {
            match String::from_utf8(bytes.clone()) {
                Ok(value) => Some(value),
                Err(err) => {
                    error!("Failed get UTF-8 string from: {}", err);
                    None
                }
            }
        }
        _ => {
            error!("Failed to get value from {}", key);
            None
        }
    }
}

async fn send_verification(config: Config, receiver: Receiver<VerificationMessage>) {
    loop {
        info!("Wait channel");
        let msg = match receiver.recv().await {
            Ok(msg) => msg,
            Err(err) => {
                error!("Failed to receive notification: {:?}", err);
                continue;
            }
        };

        info!("Get msg");
        let template_vars = json!({"verification_url": config.verification_url,"hash": msg.hash});
        let response = Client::new()
            .post(format!("https://api.eu.mailgun.net/v3/{}/messages",
                          config.mailgun_domain))
            .basic_auth("api", Some(config.mailgun_api_key.as_str()))
            .form(&[
                ("from", &config.mailgun_from),
                ("to", &msg.email),
                ("subject", &config.mailgun_subject),
                ("template", &config.mailgun_template),
                ("h:X-Mailgun-Variables", &template_vars.to_string())
            ])
            .send().await;

        match response {
            Ok(resp) =>
                if resp.status() == StatusCode::OK {
                    info!("Sent verification email to {}", msg.email);
                } else {
                    error!("Bad HTTP response: {:?}", resp.error_for_status());
                }
            Err(err) => error!("Failed to send email {:?}", err)
        };
    }
}

async fn read_notifications(conf: Config, sender: Sender<VerificationMessage>) {
    let stream_key = conf.stream_key.as_str();
    let stream_group = conf.stream_group.as_str();
    let redis_url = conf.redis_url.as_str();

    let client = match redis::Client::open(redis_url) {
        Ok(client) => client,
        Err(err) => {
            error!("Failed to open redis URL: {:#?}", err);
            exit(-1);
        }
    };

    let mut con = match client.get_connection() {
        Ok(con) => con,
        Err(err) => {
            error!("Failed to connect with redis DB: {:?}", err);
            exit(-1);
        }
    };

    info!("Connected to redis");

    let created: Result<(), _> = con.xgroup_create(stream_key, stream_group, "$");
    if let Err(e) = created {
        warn!("Group already exists: {:?}", e)
    }

    info!("Waiting for notifications from {}", stream_key);

    let opts = StreamReadOptions::default()
        .group(stream_group, format!("{}-{}", stream_group, id()));
    loop {
        let notifications: StreamReadReply = match con
            .xread_options(&[&stream_key], &[">"], &opts) {
            Ok(notifications) => notifications,
            Err(err) => {
                error!("Failed to read stream {:?}", err);
                exit(-1);
            }
        };

        for StreamKey { ids, .. } in notifications.keys {
            for StreamId { id, map } in ids {
                debug!("Receive notification ID {}", id);
                let email = parse_field(&map, "email");
                let verification_hash = parse_field(&map, "verification_hash");

                if email.is_some() && verification_hash.is_some() {
                    let msg = VerificationMessage { email: email.unwrap(), hash: verification_hash.unwrap() };
                    let _ = sender.send(msg).await;
                } else {
                    warn!("Failed get the needed data");
                }

                match con.xack(stream_key, stream_group, &[id]) {
                    Ok(()) => debug!("Ack notification"),
                    Err(err) => warn!("Failed to ack notifications: {}", err)
                }
            }
        }

        task::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug).init().unwrap();

    let conf = match Config::init_from_env() {
        Ok(conf) => conf,
        Err(err) => {
            error!("Failed to get configuration: {:?}", err);
            exit(-1);
        }
    };

    info!("Start with configuration: {:?}", conf);

    let (sender, receiver) = bounded(5);
    let _ = join!(tokio::spawn(read_notifications(conf.clone(), sender)),
        tokio::spawn(send_verification(conf.clone(), receiver)));
}
