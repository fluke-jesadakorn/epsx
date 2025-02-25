import { z } from 'zod';

// Common validation schemas
export const userSchema = z.object({
  id: z.string(),
  email: z.string().email(),
  name: z.string().optional(),
  role: z.enum(['user', 'admin']),
  createdAt: z.string().datetime(),
});

export type User = z.infer<typeof userSchema>;

// API response types
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp?: string;
}

// Add more types as needed
