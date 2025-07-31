// ============================================================================
// MIDDLEWARE ENTRY POINT - Sub-path export for middleware functionality
// ============================================================================

// Re-export all middleware functionality from the middleware directory
export * from './middleware/index';

// ============================================================================
// IMPORT GUIDANCE - Use specific imports from this entry point:
// 
// Frontend:        import { createFrontendMiddleware } from '@epsx/auth-shared/middleware';
// Admin:           import { createAdminMiddleware } from '@epsx/auth-shared/middleware';
// Unified:         import { createUnifiedMiddleware } from '@epsx/auth-shared/middleware';
// Auth Utils:      import { getSessionToken, checkPermissionAccess } from '@epsx/auth-shared/middleware';
// Security:        import { addSecurityHeaders } from '@epsx/auth-shared/middleware';
// ============================================================================