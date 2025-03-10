export interface DatabaseConfig {
  readonly MONGODB_URI: string;
}

export interface DatabaseConnectionOptions {
  readonly autoIndex?: boolean;
  readonly autoCreate?: boolean;
  readonly retryAttempts?: number;
  readonly retryDelay?: number;
  readonly debug?: boolean;
}

export interface DatabaseConnectionEvent {
  readonly timestamp: Date;
  readonly type: 'connected' | 'disconnected' | 'reconnected' | 'error';
  readonly message: string;
  readonly error?: Error;
}

export interface DatabaseConnectionMetrics {
  readonly connectionsActive: number;
  readonly connectionsAvailable: number;
  readonly connectionsCreated: number;
  readonly connectionsReused: number;
  readonly operationsQueued: number;
}

export interface DatabaseHealthCheck {
  readonly isConnected: boolean;
  readonly lastHeartbeat?: Date;
  readonly responseTime?: number;
  readonly status: 'healthy' | 'degraded' | 'unhealthy';
  readonly metrics?: DatabaseConnectionMetrics;
}
