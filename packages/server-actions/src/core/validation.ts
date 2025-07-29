import { z } from 'zod';
import { ServerError } from './error-handler';

// Common validation schemas
export const emailSchema = z.string().email('Please enter a valid email address');
export const uuidSchema = z.string().uuid('Invalid ID format');
export const dateSchema = z.string().datetime('Invalid date format');
export const packageTierSchema = z.enum(['Bronze', 'Silver', 'Gold', 'Platinum', 'Enterprise']);
export const statusSchema = z.enum(['active', 'suspended', 'deactivated']);

// Pagination schema
export const paginationSchema = z.object({
  limit: z.number().min(1).max(100).default(20),
  offset: z.number().min(0).default(0)
});

// Date range schema
export const dateRangeSchema = z.object({
  start: dateSchema,
  end: dateSchema
}).refine(data => new Date(data.start) <= new Date(data.end), {
  message: 'Start date must be before end date'
});

// User management schemas
export const updateUserTierSchema = z.object({
  userId: uuidSchema,
  newTier: packageTierSchema,
  updatedBy: uuidSchema,
  reason: z.string().optional()
});

export const bulkUserUpdateSchema = z.object({
  userIds: z.array(uuidSchema).min(1),
  updates: z.object({
    packageTier: packageTierSchema.optional(),
    status: statusSchema.optional(),
    roles: z.array(z.string()).optional()
  }),
  updatedBy: uuidSchema,
  reason: z.string().optional()
});

export const updateUserStatusSchema = z.object({
  userId: uuidSchema,
  status: statusSchema,
  updatedBy: uuidSchema,
  reason: z.string().optional()
});

// Analytics schemas
export const analyticsFiltersSchema = z.object({
  dateRange: dateRangeSchema.optional(),
  metrics: z.array(z.string()).optional(),
  groupBy: z.string().optional(),
  userId: uuidSchema.optional()
});

export const generateReportSchema = z.object({
  reportType: z.enum(['user_activity', 'system_performance', 'revenue', 'security']),
  dateRange: dateRangeSchema,
  format: z.enum(['pdf', 'csv', 'excel']),
  filters: z.record(z.any()).optional(),
  recipients: z.array(emailSchema).optional()
});

// Settings schemas
export const updateSettingsSchema = z.object({
  category: z.string().min(1),
  settings: z.record(z.any()),
  updatedBy: uuidSchema
});

export const updateFeatureFlagSchema = z.object({
  flagName: z.string().min(1),
  enabled: z.boolean(),
  userId: uuidSchema.optional(),
  updatedBy: uuidSchema
});

// IAM schemas
export const grantPermissionSchema = z.object({
  userId: uuidSchema,
  featureId: z.string().min(1),
  permission: z.string().min(1),
  grantedBy: uuidSchema,
  expiresAt: dateSchema.optional(),
  reason: z.string().optional()
});

export const revokePermissionSchema = z.object({
  permissionId: uuidSchema,
  revokedBy: uuidSchema,
  reason: z.string().optional()
});

/**
 * Validate data against a Zod schema and throw ServerError if invalid
 */
export function validateSchema<T>(schema: z.ZodSchema<T>, data: unknown, context?: string): T {
  try {
    return schema.parse(data);
  } catch (error) {
    if (error instanceof z.ZodError) {
      const errorMessages = error.errors.map(err => 
        `${err.path.join('.')}: ${err.message}`
      ).join(', ');
      
      throw new ServerError(
        `Validation failed${context ? ` for ${context}` : ''}: ${errorMessages}`,
        400,
        'VALIDATION_ERROR',
        error.errors
      );
    }
    throw error;
  }
}

/**
 * Async version of validateSchema for server actions
 */
export async function validateSchemaAsync<T>(
  schema: z.ZodSchema<T>, 
  data: unknown, 
  context?: string
): Promise<T> {
  return validateSchema(schema, data, context);
}