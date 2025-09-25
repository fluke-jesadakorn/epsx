// WEB3-FIRST AUTHENTICATION MODULE  
// Clean, focused authentication system using Web3 wallet signatures

// NEW GROUP-BASED PERMISSION SYSTEM (replaces all legacy services)
// pub mod group_permission_service; // Removed - references non-existent tables
// pub mod web3_auto_assignment_engine; // Removed - references non-existent imports
// pub mod auth_trigger_service; // Removed - references non-existent imports

// CONSOLIDATED PERMISSION SERVICE (replaces 6+ duplicate services)
// pub mod consolidated_permission_service; // Removed - references non-existent imports

// LEGACY PERMISSION MODULES (marked for removal)
pub mod web3_permission_service;
pub mod web3_group_bridge;
// pub mod unified_permission_service; // Removed - references non-existent imports

// PERFORMANCE OPTIMIZATIONS
pub mod simplified_auth_cache;

// DYNAMIC GROUP SYSTEM MODULES
pub mod dynamic_group_rules_engine;
pub mod group_template_system;

// CORE AUTH MODULES (Web3-First)
pub mod key_manager;
pub mod permissions;
pub mod granular_permissions;
pub mod hierarchy_resolver;
pub mod policy_engine;
pub mod scopes;
pub mod cleanup;

// WEB3-FIRST AUTH EXPORTS (Security Critical)

// GROUP-BASED PERMISSION SERVICE EXPORTS (LATEST)
// pub use group_permission_service::{
//     GroupPermissionService, UserGroupMembership, PermissionGroup, Web3AssignmentRule,
//     PermissionCache as GroupPermissionCache, InMemoryPermissionCache
// }; // Removed - references non-existent tables

// WEB3 AUTO-ASSIGNMENT ENGINE EXPORTS (NEW)
// pub use web3_auto_assignment_engine::{
//     Web3AutoAssignmentEngine, BlockchainNetwork, NFTOwnership, TokenBalance, DAOMembership, Web3VerificationResult, WalletAssetVerification
// }; // Removed - references non-existent imports

// AUTH TRIGGER SERVICE EXPORTS (NEW)
// pub use auth_trigger_service::{
//     AuthTriggerService, TriggerConfig, TriggerFrequency, TriggerResult, TriggerStats
// }; // Removed - references non-existent imports

// CONSOLIDATED PERMISSION SERVICE EXPORTS (NEW)
// pub use consolidated_permission_service::{
//     ConsolidatedPermissionService, Permission, PermissionSource, PermissionCheck, PermissionResult, 
//     BlockchainConfig, PermissionCache
// }; // Removed - references non-existent imports

// LEGACY WEB3 AUTHENTICATION EXPORTS (marked for removal)
pub use web3_permission_service::{
    Web3PermissionService, PermissionInfo as Web3PermissionInfo, NFTConfig, TokenConfig, 
    DAOProposal, PermissionVerificationResult
};

// WEB3 GROUP BRIDGE EXPORTS (NEW)
pub use web3_group_bridge::{
    Web3GroupBridge, Web3GroupRule, GroupAssignmentResult, GroupAssignment
};

// UNIFIED PERMISSION SERVICE EXPORTS
// pub use unified_permission_service::{
//     UnifiedPermissionService, UnifiedPermission, PermissionSource as UnifiedPermissionSource, 
//     PermissionCheck as UnifiedPermissionCheck, PermissionResult as UnifiedPermissionResult, 
//     BulkPermissionCheck, BulkPermissionResult, AccessLevel,
//     GrantPermissionRequest, PermissionStats as UnifiedPermissionStats
// }; // Removed - references non-existent imports

// PERFORMANCE OPTIMIZATION EXPORTS
pub use simplified_auth_cache::{
    SimplifiedAuthCache, SimplifiedCacheConfig, SimplifiedCacheStats,
    PermissionCacheEntry as SimplifiedPermissionCacheEntry, 
    ChallengeCacheEntry as SimplifiedChallengeCacheEntry
};

// CORE AUTH EXPORTS (Web3-First)
pub use key_manager::KeyManager;
pub use permissions::{Permission as LegacyPermission, UserClaims, check_permission_access, PermissionError, require_permission_pure, PermissionSets};
pub use granular_permissions::{
    GranularPermissionClaim, PermissionSource as GranularPermissionSource, GranularPermissionSet, 
    PermissionValidationResult, ValidationContext, GranularPermissionError
};
pub use hierarchy_resolver::{
    HierarchyResolver, PermissionHierarchy, InheritanceType, HierarchyResolution, 
    InheritanceChain, PermissionCache as HierarchyPermissionCache, HierarchyStats
};
pub use policy_engine::{
    PolicyEngine, DynamicPolicy, PolicyCondition, PolicyAction, PolicyDecision,
    PolicyEvaluationContext, PolicyTemplate, PolicyEvaluationResult
};

pub use scopes::{ScopeService, Scope, ValidatedScopes, ScopeError, SCOPE_SERVICE};
pub use cleanup::{TokenCleanupService, CleanupConfig, CleanupResult, CleanupError, start_cleanup_service, manual_cleanup, get_cleanup_stats};

// DYNAMIC GROUP SYSTEM EXPORTS
pub use dynamic_group_rules_engine::{
    DynamicGroupRulesEngine, DynamicRule, RuleCondition, LogicOperator, 
    RuleActions, RuleEvaluationResult, ConditionOperator, RuleType,
    UserContext, UserBehavioralData, Web3UserData, EvaluationContext
};
pub use group_template_system::{
    GroupTemplateSystem, GroupTemplate, TemplateCategory, EvaluationFrequency,
    TemplateParameters, ParameterDefinition, ParameterType, ValidationRule
};
