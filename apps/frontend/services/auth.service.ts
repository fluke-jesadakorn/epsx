interface AuthConfig {
  apiUrl: string;
  endpoints: {
    login: string;
    register: string;
    logout: string;
    refresh: string;
    me: string;
  };
  tokenKey: string;
  refreshTokenKey: string;
  tokenType: string;
}

interface ApiClient {
  get: (url: string) => Promise<any>;
  post: (url: string, data: any) => Promise<any>;
}

export function createAuthService(config: AuthConfig, apiClient: ApiClient) {
  const login = async (credentials: { email: string; password: string }) => {
    const response = await apiClient.post(config.endpoints.login, credentials);
    return response.data;
  };

  const register = async (userData: { email: string; password: string }) => {
    const response = await apiClient.post(config.endpoints.register, userData);
    return response.data;
  };

  const logout = async () => {
    const response = await apiClient.post(config.endpoints.logout, {});
    return response.data;
  };

  const getMe = async () => {
    const response = await apiClient.get(config.endpoints.me);
    return response.data;
  };

  return {
    login,
    register,
    logout,
    getMe
  };
}
