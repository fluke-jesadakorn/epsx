// ============================================================================
// CONSOLIDATED SHARED CORE - Single Entry Point for Core Utilities
// ============================================================================

// Error handling (most commonly used)
export * from './error-handling';
export type { Result } from './error-handling/types';

// Environment detection
export * from './environment';

// Validation (specific exports to avoid conflicts)
export { Validator, CommonSchemas } from './validation';
export type { 
  ValidationResult, 
  ValidationError as ValidationErrorType, 
  Schema, 
  ValidatedData 
} from './validation';

// ============================================================================
// IMPORT GUIDANCE - Use specific imports to minimize dependencies:
// 
// Error handling:  import { ErrorHandler } from '@epsx/shared-core';
// Environment:     import { Environment, getApiBaseUrl } from '@epsx/shared-core';
// Validation:      import { Validator, CommonSchemas } from '@epsx/shared-core';
// Types only:      import type { Result, ValidationResult } from '@epsx/shared-core';
// 
// Avoid: import * from '@epsx/shared-core' (pulls entire package)
// ============================================================================