// Get User By Firebase UID Query
// Query to retrieve a user by their Firebase UID

use crate::domain::user_management::value_objects::{FirebaseUid, Email, Permission};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserByFirebaseUidQuery {
    pub firebase_uid: FirebaseUid,
}

impl GetUserByFirebaseUidQuery {
    pub fn new(firebase_uid: FirebaseUid) -> Self {
        Self { firebase_uid }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserByFirebaseUidResponse {
    pub firebase_uid: FirebaseUid,
    pub email: Email,
    pub email_verified: bool,
    pub is_active: bool,
    pub permissions: HashSet<Permission>,
}

impl GetUserByFirebaseUidResponse {
    pub fn new(
        firebase_uid: FirebaseUid,
        email: Email,
        email_verified: bool,
        is_active: bool,
        permissions: HashSet<Permission>,
    ) -> Self {
        Self {
            firebase_uid,
            email,
            email_verified,
            is_active,
            permissions,
        }
    }
}