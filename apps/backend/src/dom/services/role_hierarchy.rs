// Role hierarchy domain service

use crate::dom::values::Role;

pub struct RoleHierarchy;

impl RoleHierarchy {
    pub fn can_upgrade(current: &Role, target: &Role) -> bool {
        current.hierarchy_level() < target.hierarchy_level()
    }
    
    pub fn get_highest(roles: &std::collections::HashSet<Role>) -> Role {
        roles.iter()
            .max_by_key(|role| role.hierarchy_level())
            .cloned()
            .unwrap_or(Role::User)
    }
    
    pub fn get_all_subordinate_roles(role: &Role) -> Vec<Role> {
        let mut subordinates = Vec::new();
        let level = role.hierarchy_level();
        
        for potential_role in [Role::User, Role::Premium, Role::Moderator, Role::Admin, Role::SuperAdmin] {
            if potential_role.hierarchy_level() < level {
                subordinates.push(potential_role);
            }
        }
        
        subordinates
    }
    
    pub fn get_next_role(current: &Role) -> Option<Role> {
        match current {
            Role::Free => Some(Role::User),
            Role::User => Some(Role::Premium),
            Role::Premium => Some(Role::Moderator),
            Role::Moderator => Some(Role::Admin),
            Role::Admin => Some(Role::SuperAdmin),
            Role::SuperAdmin => None,
            Role::ApiClient => None, // API clients cannot be upgraded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    
    #[test]
    fn should_check_upgrade_capability() {
        assert!(RoleHierarchy::can_upgrade(&Role::User, &Role::Premium));
        assert!(RoleHierarchy::can_upgrade(&Role::Premium, &Role::Admin));
        assert!(!RoleHierarchy::can_upgrade(&Role::Admin, &Role::User));
        assert!(!RoleHierarchy::can_upgrade(&Role::SuperAdmin, &Role::Admin));
    }
    
    #[test]
    fn should_get_highest_role() {
        let mut roles = HashSet::new();
        roles.insert(Role::User);
        roles.insert(Role::Premium);
        roles.insert(Role::Admin);
        
        assert_eq!(RoleHierarchy::get_highest(&roles), Role::Admin);
    }
    
    #[test]
    fn should_get_subordinate_roles() {
        let subordinates = RoleHierarchy::get_all_subordinate_roles(&Role::Admin);
        
        assert!(subordinates.contains(&Role::User));
        assert!(subordinates.contains(&Role::Premium));
        assert!(subordinates.contains(&Role::Moderator));
        assert!(!subordinates.contains(&Role::Admin));
        assert!(!subordinates.contains(&Role::SuperAdmin));
    }
    
    #[test]
    fn should_get_next_role() {
        assert_eq!(RoleHierarchy::get_next_role(&Role::User), Some(Role::Premium));
        assert_eq!(RoleHierarchy::get_next_role(&Role::Admin), Some(Role::SuperAdmin));
        assert_eq!(RoleHierarchy::get_next_role(&Role::SuperAdmin), None);
    }
}