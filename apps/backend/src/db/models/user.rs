use mongodb::bson::{self, DateTime, Document, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub firebase_uid: String,
    pub email: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub subscription: Option<Subscription>,
    #[serde(default)]
    pub profile: UserProfile,
    pub metadata: UserMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub preferences: HashMap<String, bson::Bson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetadata {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[serde(default)]
    pub last_login: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub plan_id: String,
    pub status: String,
    pub start_date: DateTime,
    pub end_date: DateTime,
    #[serde(default)]
    pub payment_details: Option<PaymentDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDetails {
    pub platform: String,
    pub transaction_id: String,
    pub amount: f64,
    pub currency: String,
}

#[allow(dead_code)]
impl User {
    pub fn new(firebase_uid: String, email: String) -> Self {
        let now = DateTime::now();
        Self {
            id: ObjectId::new(),
            firebase_uid,
            email,
            display_name: None,
            roles: vec!["free".to_string()],
            subscription: None,
            profile: UserProfile {
                company: None,
                phone: None,
                country: None,
                preferences: HashMap::new(),
            },
            metadata: UserMetadata {
                created_at: now,
                updated_at: now,
                last_login: Some(now),
            },
        }
    }

    pub fn to_claims(&self) -> Document {
        bson::doc! {
            "roles": &self.roles,
            "subscription": self.subscription.as_ref().map(|s| &s.plan_id),
            "email": &self.email,
        }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            company: None,
            phone: None,
            country: None,
            preferences: HashMap::new(),
        }
    }
}
