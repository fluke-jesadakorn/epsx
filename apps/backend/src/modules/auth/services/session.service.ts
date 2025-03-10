import { Injectable, Logger } from '@nestjs/common';
import { TokenService } from './token.service';
import { AuthLoggerService } from './auth-logger.service';
import { FirebaseAdminService } from '../../../shared/firebase-admin';

export interface SessionInfo {
  sessionId: string;
  userId: string;
  createdAt: number;
  lastActive: number;
  ipAddress?: string;
  userAgent?: string;
  geoLocation?: {
    country?: string;
    city?: string;
    latitude?: number;
    longitude?: number;
  };
}

interface InvalidationRequest {
  reason: string;
  adminId?: string;
  metadata?: Record<string, any>;
}

@Injectable()
export class SessionService {
  private readonly logger = new Logger(SessionService.name);
  private readonly activeSessions = new Map<string, SessionInfo>();

  constructor(
    private readonly tokenService: TokenService,
    private readonly authLogger: AuthLoggerService,
    private readonly firebaseAdmin: FirebaseAdminService
  ) {}

  async createSession(
    userId: string,
    sessionId: string,
    ipAddress?: string,
    userAgent?: string,
    geoLocation?: SessionInfo['geoLocation']
  ): Promise<void> {
    const sessionInfo: SessionInfo = {
      sessionId,
      userId,
      createdAt: Date.now(),
      lastActive: Date.now(),
      ipAddress,
      userAgent,
      geoLocation
    };

    this.activeSessions.set(sessionId, sessionInfo);

    await this.authLogger.logAuthEvent({
      userId,
      action: 'login',
      status: 'success',
      sessionId,
      ipAddress,
      userAgent,
      geoLocation,
      metadata: { sessionCreated: true }
    });
  }

  async invalidateSession(
    sessionId: string,
    request: InvalidationRequest
  ): Promise<void> {
    const session = this.activeSessions.get(sessionId);
    if (!session) {
      this.logger.warn(`Attempted to invalidate non-existent session: ${sessionId}`);
      return;
    }

    // Remove session from active sessions
    this.activeSessions.delete(sessionId);

    // Revoke the associated token
    await this.tokenService.revokeToken(sessionId);

    // Log the invalidation
    await this.authLogger.logAuthEvent({
      userId: session.userId,
      action: 'invalidate',
      status: 'success',
      sessionId,
      metadata: {
        reason: request.reason,
        adminId: request.adminId,
        ...request.metadata
      }
    });

    this.logger.log(`Session invalidated: ${sessionId}, Reason: ${request.reason}`);
  }

  async invalidateAllUserSessions(
    userId: string,
    request: InvalidationRequest
  ): Promise<void> {
    // Find all sessions for the user
    const userSessions = Array.from(this.activeSessions.entries())
      .filter(([_, session]) => session.userId === userId)
      .map(([sessionId]) => sessionId);

    // Invalidate each session
    await Promise.all(
      userSessions.map(sessionId =>
        this.invalidateSession(sessionId, {
          ...request,
          metadata: {
            ...request.metadata,
            massInvalidation: true
          }
        })
      )
    );

    // Revoke all Firebase refresh tokens
    await this.tokenService.revokeAllUserTokens(userId);

    this.logger.log(`All sessions invalidated for user: ${userId}, Reason: ${request.reason}`);
  }

  async getActiveSessions(userId: string): Promise<SessionInfo[]> {
    return Array.from(this.activeSessions.values())
      .filter(session => session.userId === userId)
      .sort((a, b) => b.lastActive - a.lastActive);
  }

  async updateSessionActivity(sessionId: string): Promise<void> {
    const session = this.activeSessions.get(sessionId);
    if (session) {
      session.lastActive = Date.now();
      this.activeSessions.set(sessionId, session);
    }
  }

  async isSessionActive(sessionId: string): Promise<boolean> {
    return this.activeSessions.has(sessionId) && 
           !(await this.tokenService.isTokenRevoked(sessionId));
  }

  private cleanupInactiveSessions(): void {
    const now = Date.now();
    const inactivityThreshold = 24 * 60 * 60 * 1000; // 24 hours

    for (const [sessionId, session] of this.activeSessions.entries()) {
      if (now - session.lastActive > inactivityThreshold) {
        this.invalidateSession(sessionId, {
          reason: 'Session expired due to inactivity'
        });
      }
    }
  }
}
