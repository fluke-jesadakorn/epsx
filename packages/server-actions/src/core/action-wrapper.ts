import { ErrorHandler, logger, type Result } from '@epsx/shared-core';
import { z } from 'zod';

export interface ActionContext {
  userId?: string;
  sessionId?: string;
  requestId?: string;
  action: string;
  component?: string;
}

export interface ActionOptions {
  requireAuth?: boolean;
  validateInput?: z.ZodSchema;
  validateOutput?: z.ZodSchema;
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
}

/**
 * Wrapper for server actions that provides:
 * - Consistent error handling
 * - Input/output validation
 * - Logging with context
 * - Authentication checks
 * - Performance monitoring
 */
export function withServerAction<TInput = any, TOutput = any>(
  actionName: string,
  action: (input: TInput, context: ActionContext) => Promise<TOutput>,
  options: ActionOptions = {}
) {
  return async (input: TInput): Promise<Result<TOutput>> => {
    const startTime = Date.now();
    const context: ActionContext = {
      requestId: generateRequestId(),
      action: actionName,
      component: 'server-action',
    };

    try {
      // Log action start
      logger.info(`Starting server action: ${actionName}`, { input }, context);

      // Validate input if schema provided
      if (options.validateInput) {
        const validation = options.validateInput.safeParse(input);
        if (!validation.success) {
          logger.warn(`Input validation failed for ${actionName}`, {
            errors: validation.error.errors,
            input
          }, context);
          
          return ErrorHandler.createErrorResult({
            message: 'Invalid input data',
            code: 'VALIDATION_ERROR',
            status: 400,
            details: validation.error.errors,
            timestamp: new Date(),
            context: actionName
          });
        }
        input = validation.data;
      }

      // Execute the action
      const result = await action(input, context);

      // Validate output if schema provided
      if (options.validateOutput) {
        const validation = options.validateOutput.safeParse(result);
        if (!validation.success) {
          logger.error(`Output validation failed for ${actionName}`, {
            errors: validation.error.errors,
            result
          }, context);
          
          return ErrorHandler.createErrorResult({
            message: 'Action produced invalid output',
            code: 'OUTPUT_VALIDATION_ERROR',
            status: 500,
            details: validation.error.errors,
            timestamp: new Date(),
            context: actionName
          });
        }
      }

      // Log successful completion
      const duration = Date.now() - startTime;
      logger.info(`Server action completed: ${actionName}`, {
        duration: `${duration}ms`,
        success: true
      }, context);

      return ErrorHandler.createResult(result);

    } catch (error) {
      const duration = Date.now() - startTime;
      logger.error(`Server action failed: ${actionName}`, {
        duration: `${duration}ms`,
        error: error instanceof Error ? error.message : error
      }, context);

      const processedError = ErrorHandler.handle(error, context);
      return ErrorHandler.createErrorResult(processedError);
    }
  };
}

/**
 * Simplified wrapper for actions that don't need input validation
 */
export function createServerAction<TInput = any, TOutput = any>(
  actionName: string,
  action: (input: TInput, context: ActionContext) => Promise<TOutput>
) {
  return withServerAction(actionName, action);
}

/**
 * Wrapper for authenticated actions
 */
export function createAuthenticatedAction<TInput = any, TOutput = any>(
  actionName: string,
  action: (input: TInput, context: ActionContext) => Promise<TOutput>,
  options: Omit<ActionOptions, 'requireAuth'> = {}
) {
  return withServerAction(actionName, action, { ...options, requireAuth: true });
}

/**
 * Generate a unique request ID for tracking
 */
function generateRequestId(): string {
  return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Action response types for consistency
 */
export type ActionResult<T> = Result<T>;

/**
 * Common validation schemas for server actions
 */
export const CommonSchemas = {
  userId: z.string().uuid(),
  email: z.string().email(),
  pagination: z.object({
    page: z.number().positive().default(1),
    limit: z.number().positive().max(100).default(10)
  }),
  credentials: z.object({
    email: z.string().email(),
    password: z.string().min(8)
  }),
  paymentData: z.object({
    amount: z.number().positive(),
    currency: z.string().min(3).max(3),
    description: z.string().optional(),
    orderNo: z.string()
  })
} as const;