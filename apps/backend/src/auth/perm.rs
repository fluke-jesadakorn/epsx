use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Perm {
    rules: HashMap<String, Vec<String>>,
}

impl Perm {
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        rules.insert("admin".to_string(), vec!["*".to_string()]);
        rules.insert("user".to_string(), vec!["read".to_string()]);
        
        Self { rules }
    }
    
    pub fn check(&self, role: &str, action: &str) -> bool {
        if let Some(perms) = self.rules.get(role) {
            perms.contains(&"*".to_string()) || perms.contains(&action.to_string())
        } else {
            false
        }
    }
    
    pub fn get(&self, role: &str) -> Vec<String> {
        self.rules.get(role).cloned().unwrap_or_default()
    }
}