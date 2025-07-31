// Base interfaces and errors
export * from "./interfaces/base-repository";

// Domain-specific repository interfaces
export * from "./interfaces/user-repository";
export * from "./interfaces/payment-repository";
export * from "./interfaces/permission-repository";

// Repository factory and convenience functions
export * from "./repositories/repository-factory";

// Client implementations
export { ClientUserRepository } from "./repositories/client/user-repository";
export { ClientPaymentRepository } from "./repositories/client/payment-repository";
export { ClientPermissionRepository } from "./repositories/client/permission-repository";

// Server implementations  
export { ServerUserRepository } from "./repositories/server/user-repository";
export { ServerPaymentRepository } from "./repositories/server/payment-repository";
export { ServerPermissionRepository } from "./repositories/server/permission-repository";