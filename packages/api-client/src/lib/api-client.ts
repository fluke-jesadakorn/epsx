import { AxiosInstance, AxiosError } from "axios";
import type { ChatRequest, ChatResponse } from "@epsx/types";
import { createAxiosInstance } from "./axios-instance";

export class ApiClient {
  private axios: AxiosInstance;

  constructor(baseURL?: string) {
    this.axios = createAxiosInstance(baseURL);
  }

  // Generic request methods
  async get<T>(endpoint: string, headers = {}) {
    const response = await this.axios.get<T>(endpoint, { headers });
    return response.data;
  }

  async post<T, TData = unknown>(endpoint: string, data: TData, headers = {}) {
    const response = await this.axios.post<T>(endpoint, data, { headers });
    return response.data;
  }

  async put<T, TData = unknown>(endpoint: string, data: TData, headers = {}) {
    const response = await this.axios.put<T>(endpoint, data, { headers });
    return response.data;
  }

  async delete<T>(endpoint: string, headers = {}) {
    const response = await this.axios.delete<T>(endpoint, { headers });
    return response.data;
  }

  // Specific API endpoints
  async textQuery(data: ChatRequest) {
    return this.post<ChatResponse>("/text-query", data);
  }

  // Error handling wrapper
  private async handleRequest<T>(request: Promise<T>): Promise<T> {
    try {
      return await request;
    } catch (error: unknown) {
      if (error instanceof AxiosError) {
        const message = error.response?.data?.message || error.message || "Server error occurred";
        throw new Error(message);
      }
      throw error;
    }
  }

  // Configuration methods
  setAuthToken(token: string) {
    this.axios.defaults.headers.common["Authorization"] = `Bearer ${token}`;
  }

  clearAuthToken() {
    delete this.axios.defaults.headers.common["Authorization"];
  }
}
