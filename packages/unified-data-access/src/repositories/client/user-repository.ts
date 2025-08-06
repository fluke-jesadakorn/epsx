import { NotFoundError } from "../../interfaces/base-repository";

import type { ListResult, ListOptions} from "../../interfaces/base-repository";
import type { UserRepository, User, CreateUserInput, UpdateUserInput, UserFilters } from "../../interfaces/user-repository";

export class ClientUserRepository implements UserRepository {
  private baseUrl: string;

  constructor(baseUrl: string = '/api/v1') {
    this.baseUrl = baseUrl;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const response = await fetch(url, {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    });

    if (!response.ok) {
      if (response.status === 404) {
        throw new NotFoundError('User', endpoint);
      }
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return response.json() as Promise<T>;
  }

  async get(id: string): Promise<User | null> {
    try {
      return await this.request<User>(`/users/${id}`);
    } catch (error) {
      if (error instanceof NotFoundError) {
        return null;
      }
      throw error;
    }
  }

  async getRequired(id: string): Promise<User> {
    const user = await this.get(id);
    if (!user) {
      throw new NotFoundError('User', id);
    }
    return user;
  }

  async list(filters?: UserFilters, options?: ListOptions): Promise<ListResult<User>> {
    const params = new URLSearchParams();
    
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined) {
          params.append(key, String(value));
        }
      });
    }
    
    if (options) {
      if (options.page) params.append('page', String(options.page));
      if (options.limit) params.append('limit', String(options.limit));
      if (options.sortBy) params.append('sortBy', options.sortBy);
      if (options.sortOrder) params.append('sortOrder', options.sortOrder);
    }

    return this.request<ListResult<User>>(`/users?${params.toString()}`);
  }

  async search(query: string, options?: ListOptions): Promise<ListResult<User>> {
    const params = new URLSearchParams({ q: query });
    
    if (options) {
      if (options.page) params.append('page', String(options.page));
      if (options.limit) params.append('limit', String(options.limit));
    }

    return this.request<ListResult<User>>(`/users/search?${params.toString()}`);
  }

  async create(data: CreateUserInput): Promise<User> {
    return this.request<User>('/users', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async update(id: string, data: UpdateUserInput): Promise<User> {
    return this.request<User>(`/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async delete(id: string): Promise<void> {
    await this.request(`/users/${id}`, {
      method: 'DELETE',
    });
  }

  async bulkCreate(data: CreateUserInput[]): Promise<User[]> {
    return this.request<User[]>('/users/bulk', {
      method: 'POST',
      body: JSON.stringify({ users: data }),
    });
  }

  async bulkUpdate(updates: Array<{ id: string; data: UpdateUserInput }>): Promise<User[]> {
    return this.request<User[]>('/users/bulk', {
      method: 'PUT',
      body: JSON.stringify({ updates }),
    });
  }

  async bulkDelete(ids: string[]): Promise<void> {
    await this.request('/users/bulk', {
      method: 'DELETE',
      body: JSON.stringify({ ids }),
    });
  }

  // User-specific operations
  async findByEmail(email: string): Promise<User | null> {
    try {
      return await this.request<User>(`/users/by-email/${encodeURIComponent(email)}`);
    } catch (error) {
      if (error instanceof NotFoundError) {
        return null;
      }
      throw error;
    }
  }

  async findByCredentials(email: string, password: string): Promise<User | null> {
    try {
      return await this.request<User>('/auth/validate-credentials', {
        method: 'POST',
        body: JSON.stringify({ email, password }),
      });
    } catch (error) {
      if (error instanceof NotFoundError) {
        return null;
      }
      throw error;
    }
  }

  async updatePassword(id: string, newPassword: string): Promise<void> {
    await this.request(`/users/${id}/password`, {
      method: 'PUT',
      body: JSON.stringify({ password: newPassword }),
    });
  }

  async updateLastLogin(id: string): Promise<void> {
    await this.request(`/users/${id}/last-login`, {
      method: 'PUT',
    });
  }

  async findByRole(role: string): Promise<User[]> {
    const result = await this.list({ role });
    return result.data;
  }

  async updateUserRole(id: string, role: string): Promise<User> {
    return this.update(id, { role });
  }

  async activateUser(id: string): Promise<User> {
    return this.update(id, { status: 'active' });
  }

  async deactivateUser(id: string): Promise<User> {
    return this.update(id, { status: 'inactive' });
  }

  async suspendUser(id: string, reason?: string): Promise<User> {
    return this.request<User>(`/users/${id}/suspend`, {
      method: 'PUT',
      body: JSON.stringify({ reason }),
    });
  }

  async bulkUpdateRole(userIds: string[], role: string): Promise<User[]> {
    return this.request<User[]>('/users/bulk/role', {
      method: 'PUT',
      body: JSON.stringify({ userIds, role }),
    });
  }

  async bulkUpdateStatus(userIds: string[], status: 'active' | 'inactive' | 'suspended'): Promise<User[]> {
    return this.request<User[]>('/users/bulk/status', {
      method: 'PUT',
      body: JSON.stringify({ userIds, status }),
    });
  }
}