import { Environment } from '@epsx/shared-core';

import { ClientApiClient } from './base/ClientApiClient';
import { ServerApiClient } from './base/ServerApiClient';

export class ApiClientFactory {
  private static clientInstance: ClientApiClient | null = null;
  private static serverInstance: ServerApiClient | null = null;

  static getClient(baseUrl?: string): ClientApiClient | ServerApiClient {
    if (Environment.isServer()) {
      if (!this.serverInstance) {
        this.serverInstance = new ServerApiClient(baseUrl);
      }
      return this.serverInstance;
    } else {
      if (!this.clientInstance) {
        this.clientInstance = new ClientApiClient(baseUrl);
      }
      return this.clientInstance;
    }
  }

  static getClientInstance(baseUrl?: string): ClientApiClient {
    if (!this.clientInstance) {
      this.clientInstance = new ClientApiClient(baseUrl);
    }
    return this.clientInstance;
  }

  static getServerInstance(baseUrl?: string): ServerApiClient {
    if (!this.serverInstance) {
      this.serverInstance = new ServerApiClient(baseUrl);
    }
    return this.serverInstance;
  }

  static reset(): void {
    this.clientInstance = null;
    this.serverInstance = null;
  }
}

// Default export for easy access
export const apiClient = ApiClientFactory.getClient();