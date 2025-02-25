// Common types used across the application
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

export type ErrorResponse = {
  statusCode: number;
  message: string;
  error?: string;
};

// Add more shared types as needed
