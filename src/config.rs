extern crate envconfig;

use envconfig::Envconfig;

#[derive(Envconfig)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Config {
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