import { BaseHttpClient } from '../base/BaseHttpClient';
import type {
  LoginRequest,
  RegisterRequest,
  EnhancedRegisterRequest,
  PasswordChangeRequest,
  PasswordResetRequest,
  ProfileUpdateRequest,
  UserProfile,
  RegistrationResponse,
  ApiResponse,
} from '@epsx/types';

export class AuthClient extends BaseHttpClient {
  async login(credentials: LoginRequest): Promise<ApiResponse<UserProfile>> {
    return this.post<UserProfile>('/api/auth/login', credentials);
  }

  async register(data: RegisterRequest): Promise<ApiResponse<RegistrationResponse>> {
    return this.post<RegistrationResponse>('/api/auth/register', data);
  }

  async registerEnhanced(data: EnhancedRegisterRequest): Promise<ApiResponse<RegistrationResponse>> {
    return this.post<RegistrationResponse>('/api/auth/register/enhanced', data);
  }

  async logout(): Promise<ApiResponse<void>> {
    return this.post<void>('/api/auth/logout');
  }

  async refreshToken(): Promise<ApiResponse<UserProfile>> {
    return this.post<UserProfile>('/api/auth/refresh');
  }

  async getCurrentUser(): Promise<ApiResponse<UserProfile>> {
    return this.get<UserProfile>('/api/auth/me');
  }

  async updateProfile(data: ProfileUpdateRequest): Promise<ApiResponse<UserProfile>> {
    return this.put<UserProfile>('/api/auth/profile', data);
  }

  async changePassword(data: PasswordChangeRequest): Promise<ApiResponse<void>> {
    return this.post<void>('/api/auth/change-password', data);
  }

  async resetPassword(data: PasswordResetRequest): Promise<ApiResponse<void>> {
    return this.post<void>('/api/auth/reset-password', data);
  }

  async verifyEmail(token: string): Promise<ApiResponse<void>> {
    return this.post<void>('/api/auth/verify-email', { token });
  }

  async resendVerification(email: string): Promise<ApiResponse<void>> {
    return this.post<void>('/api/auth/resend-verification', { email });
  }
}