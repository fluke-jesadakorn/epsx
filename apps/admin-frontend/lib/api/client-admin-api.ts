/**
 * Client-side Admin API for interactive components
 * Uses document.cookie to access JWT tokens in client components
 */

// Base configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'

// Client-side fetch with JWT authentication for client components
async function clientAdminFetch(endpoint: string, options: RequestInit = {}): Promise<any> {
  try {
    // Extract JWT from document.cookie (client-side)
    const token = document.cookie
      .split('; ')
      .find(row => row.startsWith('epsx_admin_jwt='))
      ?.split('=')[1]

    console.log('🔍 Client API Request:', {
      endpoint,
      hasToken: !!token,
      tokenPreview: token ? `${token.substring(0, 20)}...` : 'none'
    })

    if (!token) {
      console.warn('⚠️  No admin JWT token found in document.cookie')
    }

    const headers = {
      'Content-Type': 'application/json',
      ...(token && { 'Authorization': `Bearer ${token}` }),
      ...options.headers,
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
    })

    if (!response.ok) {
      const errorBody = await response.text().catch(() => 'Unknown error')
      console.error(`❌ Client API Error: ${response.status} ${response.statusText}`, {
        endpoint,
        body: errorBody
      })
      
      if (response.status === 401) {
        throw new Error('Authentication failed - please login again')
      }
      
      throw new Error(`API Error: ${response.status} ${response.statusText}: ${errorBody}`)
    }

    const data = await response.json()
    console.log('✅ Client API Success:', { endpoint, dataReceived: !!data })
    return data
    
  } catch (error) {
    console.error('❌ Client API Fetch Error:', {
      endpoint,
      error: error instanceof Error ? error.message : error
    })
    throw error
  }
}

// Client-side User API
export class ClientUserAPI {
  static async createUser(userData: {
    email: string;
    permissions: string[];
    display_name?: string;
  }): Promise<{ user_id: string; message: string }> {
    return await clientAdminFetch('/api/v1/admin/users', {
      method: 'POST',
      body: JSON.stringify(userData)
    })
  }

  static async updateUser(userId: string, updateData: {
    email?: string;
    permissions?: string[];
    display_name?: string; // Note: display_name is not supported by backend yet
  }): Promise<any> {
    // Filter out display_name as it's not supported by the backend AdminUpdateUserRequest
    const backendData: { email?: string; permissions?: string[] } = {}
    
    if (updateData.email) {
      backendData.email = updateData.email
    }
    if (updateData.permissions) {
      backendData.permissions = updateData.permissions
    }
    
    return await clientAdminFetch(`/api/v1/admin/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(backendData)
    })
  }

  static async deleteUser(userId: string): Promise<{ message: string }> {
    return await clientAdminFetch(`/api/v1/admin/users/${userId}`, {
      method: 'DELETE'
    })
  }

  static async searchUsers(params: {
    search?: string;
    email?: string;
    package_tier?: string;
    status?: string;
    page?: number;
    per_page?: number;
    sort_by?: string;
    sort_order?: string;
  } = {}): Promise<{
    users: any[];
    total: number;
    page: number;
    per_page: number;
  }> {
    const queryParams = new URLSearchParams()
    
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null && value !== '') {
        queryParams.append(key, value.toString())
      }
    })
    
    const queryString = queryParams.toString()
    const endpoint = `/api/v1/admin/users/search${queryString ? `?${queryString}` : ''}`
    
    return await clientAdminFetch(endpoint, {
      method: 'GET'
    })
  }
}

// Client-side Permission API
export class ClientPermissionAPI {
  static async grantEmbeddedPermission(userId: string, permissionData: {
    permission: string;
    expires_at?: string;
  }): Promise<any> {
    return await clientAdminFetch(`/api/v1/admin/users/${userId}/embedded-permissions`, {
      method: 'POST',
      body: JSON.stringify(permissionData)
    })
  }

  static async revokeEmbeddedPermission(userId: string, permission: string): Promise<any> {
    return await clientAdminFetch(`/api/v1/admin/users/${userId}/embedded-permissions/revoke`, {
      method: 'POST',
      body: JSON.stringify({ permission })
    })
  }
}