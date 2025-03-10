import { Injectable, Logger } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { AuthLog, AuthLogDocument } from '@epsx/shared';

export interface AuthEvent {
  userId?: string;
  action: 'login' | 'logout' | 'refresh' | 'invalidate' | 'token_refresh' | 'security_alert';
  status: 'success' | 'failure';
  errorCode?: string;
  errorDetails?: string;
  metadata?: Record<string, any>;
  ipAddress?: string;
  userAgent?: string;
  sessionId?: string;
  geoLocation?: {
    country?: string;
    city?: string;
    latitude?: number;
    longitude?: number;
  };
}

export interface SecurityAlert {
  type: 'suspicious_login' | 'multiple_failures' | 'session_anomaly' | 'geo_anomaly';
  severity: 'low' | 'medium' | 'high';
  userId?: string;
  description: string;
  timestamp: Date;
  metadata?: Record<string, any>;
}

@Injectable()
export class AuthLoggerService {
  private readonly logger = new Logger(AuthLoggerService.name);
  private readonly failedAttempts: Map<string, { count: number; firstAttempt: Date }> = new Map();
  private readonly recentLogins: Map<string, { timestamp: Date; location?: { lat: number; lon: number } }[]> = new Map();

  constructor(
    @InjectModel(AuthLog.name)
    private readonly logModel: Model<AuthLogDocument>
  ) {}

  async logAuthEvent(event: AuthEvent): Promise<void> {
    const logEntry = new this.logModel({
      ...event,
      timestamp: new Date(),
      isSuspicious: await this.isEventSuspicious(event)
    });

    await logEntry.save();

    // Track failed attempts
    if (event.status === 'failure' && event.ipAddress) {
      await this.trackFailedAttempt(event.ipAddress, event.userId);
    }

    // Track login patterns
    if (event.action === 'login' && event.status === 'success' && event.userId) {
      await this.trackLoginPattern(event);
    }
  }

  private async isEventSuspicious(event: AuthEvent): Promise<boolean> {
    if (event.status === 'failure') {
      // Check for multiple failures
      const recentFailures = await this.logModel.countDocuments({
        ipAddress: event.ipAddress,
        status: 'failure',
        timestamp: { $gt: new Date(Date.now() - 30 * 60 * 1000) } // Last 30 minutes
      });

      if (recentFailures >= 5) {
        await this.createSecurityAlert({
          type: 'multiple_failures',
          severity: 'high',
          userId: event.userId,
          description: `Multiple failed login attempts detected from IP ${event.ipAddress}`,
          timestamp: new Date(),
          metadata: { failureCount: recentFailures, ipAddress: event.ipAddress }
        });
        return true;
      }
    }

    if (event.action === 'login' && event.status === 'success' && event.userId && event.geoLocation) {
      // Check for unusual login locations
      const recentLogins = this.recentLogins.get(event.userId) || [];
      if (recentLogins.length > 0 && event.geoLocation.latitude && event.geoLocation.longitude) {
        const lastLogin = recentLogins[recentLogins.length - 1];
        if (lastLogin.location) {
          const distance = this.calculateDistance(
            lastLogin.location.lat,
            lastLogin.location.lon,
            event.geoLocation.latitude,
            event.geoLocation.longitude
          );

          // Alert if distance > 500km and time < 1 hour
          const timeDiff = Date.now() - lastLogin.timestamp.getTime();
          if (distance > 500 && timeDiff < 60 * 60 * 1000) {
            await this.createSecurityAlert({
              type: 'geo_anomaly',
              severity: 'high',
              userId: event.userId,
              description: 'Suspicious login detected from significantly different location',
              timestamp: new Date(),
              metadata: { distance, timeDiff, previousLocation: lastLogin.location }
            });
            return true;
          }
        }
      }
    }

    return false;
  }

  private async trackFailedAttempt(ipAddress: string, userId?: string): Promise<void> {
    const key = `${ipAddress}:${userId || 'anonymous'}`;
    const current = this.failedAttempts.get(key) || { count: 0, firstAttempt: new Date() };
    
    current.count++;
    if (current.count >= 10) {
      await this.createSecurityAlert({
        type: 'multiple_failures',
        severity: 'high',
        userId,
        description: `Excessive failed login attempts detected`,
        timestamp: new Date(),
        metadata: { ipAddress, attemptCount: current.count }
      });
    }

    this.failedAttempts.set(key, current);

    // Clear old entries after 30 minutes
    setTimeout(() => this.failedAttempts.delete(key), 30 * 60 * 1000);
  }

  private async trackLoginPattern(event: AuthEvent): Promise<void> {
    if (!event.userId || !event.geoLocation?.latitude || !event.geoLocation?.longitude) {
      return;
    }

    const userLogins = this.recentLogins.get(event.userId) || [];
    userLogins.push({
      timestamp: new Date(),
      location: {
        lat: event.geoLocation.latitude,
        lon: event.geoLocation.longitude
      }
    });

    // Keep only last 10 logins
    if (userLogins.length > 10) {
      userLogins.shift();
    }

    this.recentLogins.set(event.userId, userLogins);
  }

  private async createSecurityAlert(alert: SecurityAlert): Promise<void> {
    this.logger.warn(`Security Alert: ${alert.type} - ${alert.description}`, {
      alert,
      timestamp: new Date().toISOString()
    });

    // Store alert in log with special metadata
    await this.logAuthEvent({
      userId: alert.userId,
      action: 'security_alert',
      status: 'failure',
      metadata: {
        alertType: alert.type,
        severity: alert.severity,
        ...alert.metadata
      },
      errorDetails: alert.description
    });
  }

  async getAuthLogs(
    userId?: string,
    startDate?: Date,
    endDate?: Date,
    onlySuspicious = false
  ): Promise<AuthLogDocument[]> {
    const query: any = {};
    
    if (userId) {
      query.userId = userId;
    }
    
    if (startDate || endDate) {
      query.timestamp = {};
      if (startDate) {
        query.timestamp.$gt = startDate;
      }
      if (endDate) {
        query.timestamp.$lt = endDate;
      }
    }
    
    if (onlySuspicious) {
      query.isSuspicious = true;
    }

    return this.logModel
      .find(query)
      .sort({ timestamp: -1 })
      .limit(1000) // Limit results
      .exec();
  }

  private calculateDistance(lat1: number, lon1: number, lat2: number, lon2: number): number {
    const R = 6371; // Earth's radius in km
    const dLat = this.deg2rad(lat2 - lat1);
    const dLon = this.deg2rad(lon2 - lon1);
    const a =
      Math.sin(dLat / 2) * Math.sin(dLat / 2) +
      Math.cos(this.deg2rad(lat1)) * Math.cos(this.deg2rad(lat2)) *
      Math.sin(dLon / 2) * Math.sin(dLon / 2);
    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
    return R * c;
  }

  private deg2rad(deg: number): number {
    return deg * (Math.PI / 180);
  }
}
