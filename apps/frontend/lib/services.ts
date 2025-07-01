import { defaultAuthConfig } from '@/configs/auth.config';
import { apiClient } from '@/lib/api-client';
import { createAuthService } from '@/services/auth.service';

// Create default instances of services with standard dependencies
export const authService = createAuthService(defaultAuthConfig, apiClient);
