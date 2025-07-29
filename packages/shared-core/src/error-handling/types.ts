export interface BaseError {
  message: string;
  code: string;
  status?: number;
  details?: any;
  timestamp?: Date;
  context?: string;
}

export interface ApiError extends BaseError {
  status: number;
  endpoint?: string;
  method?: string;
}

export interface ValidationError extends BaseError {
  field?: string;
  violations?: string[];
}

export interface Result<T, E = BaseError> {
  data?: T;
  error?: E;
  success: boolean;
}

export type ErrorSeverity = 'low' | 'medium' | 'high' | 'critical';

export interface ErrorContext {
  userId?: string;
  sessionId?: string;
  requestId?: string;
  component?: string;
  action?: string;
  metadata?: Record<string, any>;
}