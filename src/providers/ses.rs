// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use std::boxed::Box;

use rusoto_core::{reactor::RequestDispatcher, Region};
use rusoto_credential::StaticProvider;
use rusoto_ses::{Content, Destination, Message, SendEmailRequest, Ses, SesClient};

use super::{Provider, ProviderError};
use settings::Settings;

pub struct SesProvider
{
  client: Box<Ses>,
  sender: String,
}

impl SesProvider
{
  pub fn new(settings: &Settings) -> SesProvider
  {
    let region = settings
      .ses
      .region
      .parse::<Region>()
      .expect("invalid region");

    let client: Box<Ses> = if let Some(ref keys) = settings.ses.keys {
      let creds = StaticProvider::new(keys.access.to_string(), keys.secret.to_string(), None, None);
      Box::new(SesClient::new(RequestDispatcher::default(), creds, region))
    } else {
      Box::new(SesClient::simple(region))
    };

    SesProvider {
      client,
      sender: settings.sender.clone(),
    }
  }
}

impl Provider for SesProvider
{
  fn send(
    &self,
    to: &str,
    cc: &[&str],
    subject: &str,
    body_text: &str,
    body_html: Option<&str>,
  ) -> Result<String, ProviderError>
  {
    let mut destination = Destination::default();
    destination.to_addresses = Some(vec![to.to_string()]);
    if cc.len() > 0 {
      destination.cc_addresses = Some(cc.iter().map(|s| s.to_string()).collect());
    }

    let mut message = Message::default();
    message.subject.data = subject.to_string();
    message.body.text = Some(Content {
      charset: None,
      data: body_text.to_string(),
    });
    if let Some(html) = body_html {
      message.body.html = Some(Content {
        charset: None,
        data: html.to_string(),
      });
    }

    let mut request = SendEmailRequest::default();
    request.destination = destination;
    request.message = message;
    request.source = self.sender.to_string();

    match self.client.send_email(&request).sync() {
      Ok(response) => Ok(response.message_id),
      Err(error) => {
        println!("SES error: {}", error);
        Err(ProviderError {
          description: error.to_string(),
        })
      }
    }
  }
}