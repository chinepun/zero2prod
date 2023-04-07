use std::fmt;

use serde::Deserialize;

use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;
use serde::{Serialize,};

#[derive(Serialize, Deserialize,)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl fmt::Display for NewSubscriber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[subscriber details => {},{}]", self.email.to_string(), self.name.to_string())
    }
}