// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::{HashMap, HashSet}, env, error::Error,
};

use super::*;

struct CleanEnvironment {
    vars_to_reinstate: HashMap<String, String>,
    keys_to_clear: HashSet<String>,
}

impl CleanEnvironment {
    pub fn new(keys: Vec<&str>) -> CleanEnvironment {
        let mut snapshot = CleanEnvironment {
            vars_to_reinstate: HashMap::new(),
            keys_to_clear: HashSet::new(),
        };

        snapshot.initialise(keys);

        snapshot
    }

    fn initialise(&mut self, keys: Vec<&str>) {
        for key in keys {
            if let Ok(value) = env::var(key) {
                self.vars_to_reinstate.insert(String::from(key), value);
                env::remove_var(key);
            } else {
                self.keys_to_clear.insert(String::from(key));
            }
        }
    }
}

impl Drop for CleanEnvironment {
    fn drop(&mut self) {
        for (key, value) in &self.vars_to_reinstate {
            env::set_var(key, &value);
        }

        for key in &self.keys_to_clear {
            env::remove_var(key);
        }
    }
}

#[test]
fn env_vars_take_precedence() {
    let _clean_env = CleanEnvironment::new(vec![
        "FXA_EMAIL_AUTHDB_BASEURI",
        "FXA_EMAIL_BOUNCELIMITS_ENABLED",
        "FXA_EMAIL_PROVIDER",
        "FXA_EMAIL_SENDER_ADDRESS",
        "FXA_EMAIL_SENDER_NAME",
        "FXA_EMAIL_SENDGRID_KEY",
        "FXA_EMAIL_SES_REGION",
        "FXA_EMAIL_SES_KEYS_ACCESS",
        "FXA_EMAIL_SES_KEYS_SECRET",
        "FXA_EMAIL_SMTP_HOST",
        "FXA_EMAIL_SMTP_PORT",
        "FXA_EMAIL_SMTP_USER",
        "FXA_EMAIL_SMTP_PASSWORD",
    ]);

    match Settings::new() {
        Ok(settings) => {
            let auth_db_base_uri = format!("{}foo/", &settings.authdb.baseuri);
            let bounce_limits_enabled = !settings.bouncelimits.enabled;
            let provider = if settings.provider == "ses" {
                "smtp"
            } else {
                "ses"
            };
            let sender_address = format!("1{}", &settings.sender.address);
            let sender_name = format!("{}1", &settings.sender.name);
            let sendgrid_api_key = String::from(
                "000000000000000000000000000000000000000000000000000000000000000000000",
            );
            let ses_region = if settings.ses.region == "us-east-1" {
                "eu-west-1"
            } else {
                "us-east-1"
            };
            let ses_keys = if let Some(ref keys) = settings.ses.keys {
                AwsKeys {
                    access: format!("{}A", keys.access),
                    secret: format!("{}s", keys.secret),
                }
            } else {
                AwsKeys {
                    access: String::from("A"),
                    secret: String::from("s"),
                }
            };
            let smtp_host = format!("{}2", &settings.smtp.host);
            let smtp_port = settings.smtp.port + 3;
            let smtp_user = if let Some(ref user) = settings.smtp.user {
                format!("{}4", user)
            } else {
                String::from("4")
            };
            let smtp_password = if let Some(ref password) = settings.smtp.password {
                format!("{}5", password)
            } else {
                String::from("5")
            };

            env::set_var("FXA_EMAIL_AUTHDB_BASEURI", &auth_db_base_uri);
            env::set_var(
                "FXA_EMAIL_BOUNCELIMITS_ENABLED",
                &bounce_limits_enabled.to_string(),
            );
            env::set_var("FXA_EMAIL_PROVIDER", &provider);
            env::set_var("FXA_EMAIL_SENDER_ADDRESS", &sender_address);
            env::set_var("FXA_EMAIL_SENDER_NAME", &sender_name);
            env::set_var("FXA_EMAIL_SENDGRID_KEY", &sendgrid_api_key);
            env::set_var("FXA_EMAIL_SES_REGION", &ses_region);
            env::set_var("FXA_EMAIL_SES_KEYS_ACCESS", &ses_keys.access);
            env::set_var("FXA_EMAIL_SES_KEYS_SECRET", &ses_keys.secret);
            env::set_var("FXA_EMAIL_SMTP_HOST", &smtp_host);
            env::set_var("FXA_EMAIL_SMTP_PORT", &smtp_port.to_string());
            env::set_var("FXA_EMAIL_SMTP_USER", &smtp_user);
            env::set_var("FXA_EMAIL_SMTP_PASSWORD", &smtp_password);

            match Settings::new() {
                Ok(env_settings) => {
                    assert_eq!(env_settings.authdb.baseuri, auth_db_base_uri);
                    assert_eq!(env_settings.bouncelimits.enabled, bounce_limits_enabled);
                    assert_eq!(env_settings.provider, provider);
                    assert_eq!(env_settings.sender.address, sender_address);
                    assert_eq!(env_settings.sender.name, sender_name);
                    assert_eq!(env_settings.ses.region, ses_region);
                    assert_eq!(env_settings.smtp.host, smtp_host);
                    assert_eq!(env_settings.smtp.port, smtp_port);

                    if let Some(env_sendgrid) = env_settings.sendgrid {
                        assert_eq!(env_sendgrid.key, sendgrid_api_key);
                    } else {
                        assert!(false, "settings.sendgrid was not set");
                    }

                    if let Some(env_keys) = env_settings.ses.keys {
                        assert_eq!(env_keys.access, ses_keys.access);
                        assert_eq!(env_keys.secret, ses_keys.secret);
                    } else {
                        assert!(false, "ses.keys were not set");
                    }

                    if let Some(env_user) = env_settings.smtp.user {
                        assert_eq!(env_user, smtp_user);
                    } else {
                        assert!(false, "smtp.user was not set");
                    }

                    if let Some(env_password) = env_settings.smtp.password {
                        assert_eq!(env_password, smtp_password);
                    } else {
                        assert!(false, "smtp.password was not set");
                    }
                }
                Err(error) => {
                    println!("{}", error);
                    assert!(false);
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            assert!(false);
        }
    }
}

#[test]
fn invalid_auth_db_base_uri() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_AUTHDB_BASEURI"]);
    env::set_var("FXA_EMAIL_AUTHDB_BASEURI", "http://example.com");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}

#[test]
fn invalid_bouncelimits_enabled() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_BOUNCELIMITS_ENABLED"]);
    env::set_var("FXA_EMAIL_BOUNCELIMITS_ENABLED", "falsey");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "invalid type"),
    }
}

#[test]
fn invalid_provider() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_PROVIDER"]);
    env::set_var("FXA_EMAIL_PROVIDER", "smtps");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}

#[test]
fn invalid_sender_address() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_SENDER_ADDRESS"]);
    env::set_var("FXA_EMAIL_SENDER_ADDRESS", "foo");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}

#[test]
fn invalid_sender_name() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_SENDER_NAME"]);
    env::set_var("FXA_EMAIL_SENDER_NAME", "foo@example.com");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}

#[test]
fn invalid_sendgrid_api_key() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_SENDGRID_KEY"]);
    env::set_var("FXA_EMAIL_SENDGRID_KEY", "foo bar");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}

#[test]
fn invalid_ses_region() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_SES_REGION"]);
    env::set_var("FXA_EMAIL_SES_REGION", "us-east-1a");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}

#[test]
fn invalid_ses_access_key() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_SES_KEYS_ACCESS"]);
    env::set_var("FXA_EMAIL_SES_KEYS_ACCESS", "DEADBEEF DEADBEEF");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}

#[test]
fn invalid_ses_secret_key() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_SES_KEYS_SECRET"]);
    env::set_var("FXA_EMAIL_SES_KEYS_SECRET", "DEADBEEF DEADBEEF");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}

#[test]
fn invalid_smtp_host() {
    let _clean_env = CleanEnvironment::new(vec!["FXA_EMAIL_SMTP_HOST"]);
    env::set_var("FXA_EMAIL_SMTP_HOST", "https://mail.google.com/");

    match Settings::new() {
        Ok(_settings) => assert!(false, "Settings::new should have failed"),
        Err(error) => assert_eq!(error.description(), "configuration error"),
    }
}
