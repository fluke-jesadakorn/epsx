use std::collections::HashMap;
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sess {
    pub id: String,
    pub uid: String,
    pub exp: i64,
}

#[derive(Debug, Clone)]
pub struct SessMgr {
    ttl: i64,
    store: HashMap<String, Sess>,
}

impl SessMgr {
    pub fn new() -> Self {
        let ttl = env::var("SESS_TTL")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);
            
        Self { 
            ttl, 
            store: HashMap::new() 
        }
    }
    
    pub fn create(&mut self, uid: String) -> Sess {
        let id = format!("sess_{}", uid);
        let exp = chrono::Utc::now().timestamp() + self.ttl;
        
        let sess = Sess { id: id.clone(), uid, exp };
        self.store.insert(id, sess.clone());
        sess
    }
    
    pub fn get(&self, id: &str) -> Option<&Sess> {
        self.store.get(id)
    }
    
    pub fn del(&mut self, id: &str) {
        self.store.remove(id);
    }
    
    pub fn valid(&self, sess: &Sess) -> bool {
        chrono::Utc::now().timestamp() < sess.exp
    }
}