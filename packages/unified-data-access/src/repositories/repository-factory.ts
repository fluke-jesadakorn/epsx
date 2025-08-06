
// Server-side implementations (would make direct API calls or database queries)
import { ClientPaymentRepository } from "./client/payment-repository";
import { ClientPermissionRepository } from "./client/permission-repository";
import { ClientUserRepository } from "./client/user-repository";
import { ServerPaymentRepository } from "./server/payment-repository";
import { ServerPermissionRepository } from "./server/permission-repository";
// Client-side implementations (would make HTTP requests through API client)
import { ServerUserRepository } from "./server/user-repository";

import type { RepositoryFactory, RepositoryContext } from "../interfaces/base-repository";
import type { PaymentRepository } from "../interfaces/payment-repository";
import type { PermissionRepository } from "../interfaces/permission-repository";
import type { UserRepository } from "../interfaces/user-repository";

export class UnifiedRepositoryFactory implements RepositoryFactory {
  private context: RepositoryContext;
  private baseUrl?: string;
  
  // Repository instances (singleton pattern)
  private userRepository?: UserRepository;
  private paymentRepository?: PaymentRepository;
  private permissionRepository?: PermissionRepository;

  constructor(context: RepositoryContext, options?: { baseUrl?: string }) {
    this.context = context;
    this.baseUrl = options?.baseUrl;
  }

  getUserRepository(): UserRepository {
    if (!this.userRepository) {
      this.userRepository = this.context === 'server' 
        ? new ServerUserRepository()
        : new ClientUserRepository(this.baseUrl);
    }
    return this.userRepository;
  }

  getPaymentRepository(): PaymentRepository {
    if (!this.paymentRepository) {
      this.paymentRepository = this.context === 'server'
        ? new ServerPaymentRepository()
        : new ClientPaymentRepository(this.baseUrl);
    }
    return this.paymentRepository;
  }

  getPermissionRepository(): PermissionRepository {
    if (!this.permissionRepository) {
      this.permissionRepository = this.context === 'server'
        ? new ServerPermissionRepository()
        : new ClientPermissionRepository(this.baseUrl);
    }
    return this.permissionRepository;
  }

  getAnalyticsRepository(): unknown {
    // TODO: Implement analytics repository
    throw new Error("Analytics repository not yet implemented");
  }
}

// Global factory instances
let serverFactory: UnifiedRepositoryFactory;
let clientFactory: UnifiedRepositoryFactory;

// Factory access functions
export function getServerRepositories(): UnifiedRepositoryFactory {
  if (!serverFactory) {
    serverFactory = new UnifiedRepositoryFactory('server');
  }
  return serverFactory;
}

export function getClientRepositories(baseUrl?: string): UnifiedRepositoryFactory {
  if (!clientFactory) {
    clientFactory = new UnifiedRepositoryFactory('client', { baseUrl });
  }
  return clientFactory;
}

// Convenience functions for direct repository access
export function getUserRepository(context: RepositoryContext = 'client', baseUrl?: string): UserRepository {
  const factory = context === 'server' ? getServerRepositories() : getClientRepositories(baseUrl);
  return factory.getUserRepository();
}

export function getPaymentRepository(context: RepositoryContext = 'client', baseUrl?: string): PaymentRepository {
  const factory = context === 'server' ? getServerRepositories() : getClientRepositories(baseUrl);
  return factory.getPaymentRepository();
}

export function getPermissionRepository(context: RepositoryContext = 'client', baseUrl?: string): PermissionRepository {
  const factory = context === 'server' ? getServerRepositories() : getClientRepositories(baseUrl);
  return factory.getPermissionRepository();
}