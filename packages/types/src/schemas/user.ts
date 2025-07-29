import { z } from 'zod';

export const UserPreferencesSchema = z.object({
  theme: z.enum(['light', 'dark', 'system']).optional(),
  language: z.string().optional(),
  timezone: z.string().optional(),
  notifications: z.object({
    email: z.boolean(),
    push: z.boolean(),
    sms: z.boolean(),
    marketing: z.boolean(),
    updates: z.boolean(),
  }).optional(),
});

export const UserProfileSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  name: z.string().optional(),
  displayName: z.string().optional(),
  avatar: z.string().url().optional(),
  role: z.string(),
  isActive: z.boolean(),
  createdAt: z.date(),
  updatedAt: z.date(),
  preferences: UserPreferencesSchema.optional(),
});

export const LoginRequestSchema = z.object({
  type: z.literal('credentials'),
  email: z.string().email(),
  password: z.string().min(8),
});

export const RegisterRequestSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
  name: z.string().optional(),
  package_tier: z.string().optional(),
});

export const EnhancedRegisterRequestSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
  package_tier: z.string(),
  referral_code: z.string().optional(),
  source: z.string(),
  region: z.string().optional(),
  utm_source: z.string().optional(),
  utm_campaign: z.string().optional(),
});

export const PasswordChangeRequestSchema = z.object({
  currentPassword: z.string(),
  newPassword: z.string().min(8),
});

export const PasswordResetRequestSchema = z.object({
  email: z.string().email(),
});

export const ProfileUpdateRequestSchema = z.object({
  name: z.string().optional(),
  displayName: z.string().optional(),
  avatar: z.string().url().optional(),
  preferences: UserPreferencesSchema.partial().optional(),
});