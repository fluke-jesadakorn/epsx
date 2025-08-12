'use client';

// Module Authentication Context - Enhanced Multi-Provider Version
// This file now exports the enhanced module auth system with Casbin integration while maintaining backward compatibility

// Re-export everything from the enhanced module context
export {
  EnhancedModuleAuthProvider as ModuleAuthProvider,
  useModuleAuth,
  useEnhancedModuleAuth,
  useAdminPermissions,
  ModuleAccessStatus,
  withEnhancedModuleAccess
} from './enhanced-module-ctx';

// Legacy export for backward compatibility
export { useModuleAuth as default };