use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use futures::lock::Mutex;

use crate::payload::{Pin, ReplyToken};

/// Store in which payloads are temporarily saved.
#[derive(Clone, Default)]
pub(super) struct Store {
    inner: Arc<Mutex<StoreInner>>,
    config: super::ServerConfig,
}

impl Store {
    pub(super) fn new(config: super::ServerConfig) -> Store {
        Store {
            inner: Default::default(),
            config,
        }
    }

    pub(super) async fn create(
        &self,
        data: String,
        reply_pin: Option<Pin>,
        reply_token: Option<ReplyToken>,
    ) -> Result<(Pin, DateTime<Utc>), super::RequestError> {
        let mut inner = self.inner.lock().await;

        if inner.payloads.len() > self.config.max_payloads {
            return Err(super::RequestError::Full);
        }

        let expiration_duration = chrono::Duration::from_std(self.config.payload_expiration)
            .expect("Couldn't convert expiration to chrono Duration");
        let expiration = Utc::now() + expiration_duration;

        let pin = inner.next_id();
        inner.payloads.insert(
            pin,
            StoredPayload {
                data,
                reply_pin,
                reply_token,
                expiration,
            },
        );

        Ok((pin, expiration))
    }

    pub(super) async fn get(&self, id: Pin) -> Option<StoredPayload> {
        let mut inner = self.inner.lock().await;
        let payload = inner.payloads.remove(&id)?;
        Some(payload)
    }

    pub(super) async fn push_reply(
        &self,
        pin: Pin,
        token: ReplyToken,
        data: String,
        reply_pin: Option<Pin>,
        reply_token: Option<ReplyToken>,
    ) -> Result<DateTime<Utc>, super::RequestError> {
        let mut inner = self.inner.lock().await;

        if inner.payloads.len() > self.config.max_payloads {
            return Err(super::RequestError::Full);
        }

        {
            // validate that the reply pin has right token, and remove it if it is
            match inner.replies.get(&pin) {
                Some(info) if info.token == token => {}
                _ => {
                    return Err(super::RequestError::InvalidReply);
                }
            }
            inner.replies.remove(&pin);
        }

        let expiration_duration = chrono::Duration::from_std(self.config.payload_expiration)
            .expect("Couldn't convert expiration to chrono Duration");
        let expiration = Utc::now() + expiration_duration;

        inner.payloads.insert(
            pin,
            StoredPayload {
                data,
                reply_pin,
                reply_token,
                expiration,
            },
        );

        Ok(expiration)
    }

    pub(super) async fn get_reply_token(&self) -> (Pin, ReplyToken) {
        let mut inner = self.inner.lock().await;

        let expiration_duration = chrono::Duration::from_std(self.config.reply_expiration)
            .expect("Couldn't convert expiration to chrono Duration");
        let expiration = Utc::now() + expiration_duration;

        let pin = inner.next_id();
        let token = rand::random();
        inner.replies.insert(pin, ReplyInfo { expiration, token });

        (pin, token)
    }

    pub(super) async fn cleanup(&self) {
        let mut inner = self.inner.lock().await;
        let now = Utc::now();

        {
            // cleanup payloads
            let mut expired = HashSet::new();
            for (id, payload) in &inner.payloads {
                if payload.expiration < now {
                    expired.insert(*id);
                }
            }
            for id in expired {
                inner.payloads.remove(&id);
            }
        }

        {
            // cleanup reply tokens
            let mut expired = HashSet::new();
            for (id, reply) in &inner.replies {
                if reply.expiration < now {
                    expired.insert(*id);
                }
            }
            for id in expired {
                inner.replies.remove(&id);
            }
        }
    }
}

#[derive(Default)]
struct StoreInner {
    /// Payloads that are waiting to be fetched.
    payloads: HashMap<Pin, StoredPayload>,

    /// Payloads reply information.
    replies: HashMap<Pin, ReplyInfo>,
}

impl StoreInner {
    fn next_id(&self) -> Pin {
        loop {
            let pin = Pin::generate();
            if !self.payloads.contains_key(&pin) && !self.replies.contains_key(&pin) {
                return pin;
            }
        }
    }
}

pub struct StoredPayload {
    pub data: String,
    pub reply_pin: Option<Pin>,
    pub reply_token: Option<ReplyToken>,
    expiration: DateTime<Utc>,
}

/// Payload reply information.
///
/// When a payload gets created and expects a reply, from the consumer
/// a reply pin and a unique random token. The latter is used to make
/// sure that it's not possible to push a payload to the reply pin without
/// being the consumer of the original pin.
pub struct ReplyInfo {
    expiration: DateTime<Utc>,
    token: ReplyToken,
}
