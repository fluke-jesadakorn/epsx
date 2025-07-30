'use server';

import { createServerAction } from '../core/action-wrapper';
import { serverGet, serverPost, serverDelete } from '../core/enhanced-request';
import { z } from 'zod';

// Schema definitions
const ModuleAssignmentSchema = z.object({
  userId: z.string().min(1),
  moduleId: z.string().min(1),
  accessLevel: z.enum(['bronze', 'silver', 'gold', 'platinum', 'enterprise']),
  expiresAt: z.string().datetime().optional()
});

const ApiKeyCreationSchema = z.object({
  name: z.string().min(1),
  modules: z.array(z.string()).min(1),
  accessLevel: z.enum(['bronze', 'silver', 'gold', 'platinum', 'enterprise']),
  expiresAt: z.string().datetime().optional()
});

const BulkModuleAssignmentSchema = z.object({
  userId: z.string().min(1),
  moduleAssignments: z.array(z.object({
    moduleId: z.string().min(1),
    accessLevel: z.enum(['bronze', 'silver', 'gold', 'platinum', 'enterprise']),
    expiresAt: z.string().datetime().optional()
  }))
});

// Get all available modules
export const getModules = createServerAction(
  'modules.getModules',
  async (_, context) => {
    const data = await serverGet(`/admin/modules`, {
      action: context.action || 'modules.getModules',
      userId: context.userId || 'unknown',
      requestId: context.requestId || 'unknown'
    });

    return {
      success: true,
      data: data.modules || []
    };
  }
);

// Get user's module assignments
export const getUserModuleAssignments = createServerAction(
  'modules.getUserModuleAssignments',
  async (input: { userId: string }, context) => {
    const data = await serverGet(`/admin/users/${input.userId}/modules`, {
      action: context.action || 'modules.getUserModuleAssignments',
      userId: context.userId || 'unknown',
      requestId: context.requestId || 'unknown'
    });

    return {
      success: true,
      data: data.modules || []
    };
  }
);

// Assign modules to a user
export const assignModulesToUser = createServerAction(
  'modules.assignModulesToUser',
  async (input: { userId: string; moduleAssignments: Array<{ moduleId: string; accessLevel: string; expiresAt?: string }> }, context) => {
    const data = await serverPost(`/admin/users/${input.userId}/modules`, {
      assignments: input.moduleAssignments
    }, {
      action: context.action || 'modules.assignModulesToUser',
      userId: context.userId || 'unknown',
      requestId: context.requestId || 'unknown'
    });

    return {
      success: true,
      data: data.assignments || []
    };
  }
);

// Revoke module access from a user
export const revokeModuleAccess = createServerAction(
  'modules.revokeModuleAccess',
  async (input: { userId: string; moduleId: string }, context) => {
    await serverDelete(`/admin/users/${input.userId}/modules/${input.moduleId}`, {
      action: context.action || 'modules.revokeModuleAccess',
      userId: context.userId || 'unknown',
      requestId: context.requestId || 'unknown'
    });

    return {
      success: true,
      message: 'Module access revoked successfully'
    };
  }
);

// Create API key
export const createApiKey = createServerAction(
  'modules.createApiKey',
  async (input: { name: string; modules: string[]; accessLevel: string; expiresAt?: string }, context) => {
    const data = await serverPost(`/admin/api-keys`, {
      name: input.name,
      modules: input.modules,
      access_level: input.accessLevel,
      expires_at: input.expiresAt
    }, {
      action: context.action || 'modules.createApiKey',
      userId: context.userId || 'unknown',
      requestId: context.requestId || 'unknown'
    });

    return {
      success: true,
      data: data.api_key || {}
    };
  }
);

// List API keys
export const listApiKeys = createServerAction(
  'modules.listApiKeys',
  async (input: { limit?: number; offset?: number } = {}, context) => {
    const { limit = 50, offset = 0 } = input;
    const params = new URLSearchParams({
      limit: limit.toString(),
      offset: offset.toString()
    });

    const data = await serverGet(`/admin/api-keys?${params}`, {
      action: context.action || 'modules.listApiKeys',
      userId: context.userId || 'unknown',
      requestId: context.requestId || 'unknown'
    });

    return {
      success: true,
      data: data.api_keys || [],
      total: data.total || 0
    };
  }
);

// Revoke API key
export const revokeApiKey = createServerAction(
  'modules.revokeApiKey',
  async (input: { keyId: string }, context) => {
    await serverDelete(`/admin/api-keys/${input.keyId}`, {
      action: context.action || 'modules.revokeApiKey',
      userId: context.userId || 'unknown',
      requestId: context.requestId || 'unknown'
    });

    return {
      success: true,
      message: 'API key revoked successfully'
    };
  }
);

// Get module usage analytics
export const getModuleUsageAnalytics = createServerAction(
  'modules.getModuleUsageAnalytics',
  async (input: { moduleId?: string; startDate?: string; endDate?: string; groupBy?: string } = {}, context) => {
    const { moduleId, startDate, endDate, groupBy = 'day' } = input;
    const params = new URLSearchParams();
    if (moduleId) params.set('module_id', moduleId);
    if (startDate) params.set('start_date', startDate);
    if (endDate) params.set('end_date', endDate);
    params.set('group_by', groupBy);

    const data = await serverGet(`/admin/analytics/modules?${params}`, {
      action: context.action || 'modules.getModuleUsageAnalytics',
      userId: context.userId || 'unknown',
      requestId: context.requestId || 'unknown'
    });

    return {
      success: true,
      data: data.analytics || {}
    };
  }
);

// Export types for TypeScript support
export type ModuleAssignment = z.infer<typeof ModuleAssignmentSchema>;
export type ApiKeyCreation = z.infer<typeof ApiKeyCreationSchema>;
export type BulkModuleAssignment = z.infer<typeof BulkModuleAssignmentSchema>;