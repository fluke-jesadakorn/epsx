import axios from "axios";

// Default config for axios instance
const defaultConfig = {
  timeout: 10000,
  headers: {
    "Content-Type": "application/json",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    "X-Source": "Cloudflare-Workers",
  },
  withCredentials: true,
};

export const createAxiosInstance = (baseURL?: string) => {
  const instance = axios.create({
    ...defaultConfig,
    baseURL: baseURL || process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001",
  });

  // Request interceptor
  instance.interceptors.request.use(
    (config) => {
      // TODO: Add authentication token to headers
      // if (token) {
      //   config.headers.Authorization = `Bearer ${token}`;
      // }
      return config;
    },
    (error) => Promise.reject(error)
  );

  // Response interceptor
  instance.interceptors.response.use(
    (response) => response,
    async (error) => {
      const originalRequest = error.config;

      // TODO: Handle token refresh
      if (error.response?.status === 401 && !originalRequest._retry) {
        originalRequest._retry = true;
        // try {
        //   const newToken = await refreshToken();
        //   originalRequest.headers.Authorization = `Bearer ${newToken}`;
        //   return instance(originalRequest);
        // } catch (refreshError) {
        //   return Promise.reject(refreshError);
        // }
      }

      return Promise.reject(error);
    }
  );

  return instance;
};
