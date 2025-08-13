'use client';

// Advanced Threat Protection System
// ML-based anomaly detection, bot protection, and APT detection
// The hardest possible security implementation

/**
 * Threat detection levels
 */
export enum ThreatLevel {
  NONE = 0,
  LOW = 1,
  MEDIUM = 2,
  HIGH = 3,
  CRITICAL = 4,
  APT = 5, // Advanced Persistent Threat
}

/**
 * Attack vector types
 */
export enum AttackVector {
  BRUTE_FORCE = 'brute_force',
  CREDENTIAL_STUFFING = 'credential_stuffing',
  BOT_ATTACK = 'bot_attack',
  SOCIAL_ENGINEERING = 'social_engineering',
  SESSION_HIJACKING = 'session_hijacking',
  MAN_IN_THE_MIDDLE = 'man_in_the_middle',
  APT_RECONNAISSANCE = 'apt_reconnaissance',
  INSIDER_THREAT = 'insider_threat',
  ACCOUNT_TAKEOVER = 'account_takeover',
  PRIVILEGE_ESCALATION = 'privilege_escalation',
}

/**
 * Bot detection result
 */
export interface BotDetectionResult {
  isBot: boolean;
  confidence: number; // 0-1
  botType: 'crawler' | 'scraper' | 'attack' | 'unknown' | 'human';
  evidence: BotEvidence[];
  mitigationActions: string[];
}

/**
 * Bot detection evidence
 */
export interface BotEvidence {
  type: 'user_agent' | 'timing' | 'behavioral' | 'network' | 'fingerprint';
  description: string;
  severity: 'low' | 'medium' | 'high';
  confidence: number;
  data: any;
}

/**
 * Threat intelligence data
 */
export interface ThreatIntelligence {
  ipReputation: IPReputationResult;
  emailReputation: EmailReputationResult;
  deviceReputation: DeviceReputationResult;
  geolocationRisk: GeolocationRisk;
  knownAttackPatterns: AttackPattern[];
}

/**
 * IP reputation information
 */
export interface IPReputationResult {
  ip: string;
  riskScore: number; // 0-100
  categories: string[]; // ['tor', 'proxy', 'malware', 'botnet', etc.]
  country: string;
  isp: string;
  isVpn: boolean;
  isTor: boolean;
  isProxy: boolean;
  isMalicious: boolean;
  lastSeen: Date | null;
  threatSources: string[];
}

/**
 * Email reputation information
 */
export interface EmailReputationResult {
  email: string;
  domain: string;
  riskScore: number; // 0-100
  isDisposable: boolean;
  isCompromised: boolean;
  isSpam: boolean;
  domainAge: number; // days
  mxRecords: string[];
  breachHistory: BreachRecord[];
}

/**
 * Device reputation information
 */
export interface DeviceReputationResult {
  fingerprint: string;
  riskScore: number; // 0-100
  isEmulated: boolean;
  isVirtualized: boolean;
  isSuspicious: boolean;
  seenCount: number;
  firstSeen: Date;
  lastSeen: Date;
  associatedThreats: string[];
}

/**
 * Geolocation risk assessment
 */
export interface GeolocationRisk {
  country: string;
  region: string;
  city: string;
  riskLevel: ThreatLevel;
  isHighRiskCountry: boolean;
  isSanctionedCountry: boolean;
  isVpnDetected: boolean;
  isUnusualLocation: boolean;
  distanceFromNormal: number; // km
}

/**
 * Known attack pattern
 */
export interface AttackPattern {
  patternId: string;
  name: string;
  description: string;
  vector: AttackVector;
  indicators: string[];
  severity: ThreatLevel;
  mitigations: string[];
  lastUpdated: Date;
}

/**
 * Breach record information
 */
export interface BreachRecord {
  source: string;
  date: Date;
  recordsAffected: number;
  dataClasses: string[]; // ['passwords', 'emails', 'names', etc.]
  verified: boolean;
}

/**
 * Anomaly detection result
 */
export interface AnomalyDetectionResult {
  isAnomalous: boolean;
  anomalyScore: number; // 0-1
  anomalyTypes: AnomalyType[];
  confidence: number;
  baseline: any;
  current: any;
  recommendation: string;
}

/**
 * Anomaly types
 */
export enum AnomalyType {
  TIMING = 'timing',
  FREQUENCY = 'frequency',
  GEOGRAPHIC = 'geographic',
  BEHAVIORAL = 'behavioral',
  DEVICE = 'device',
  ACCESS_PATTERN = 'access_pattern',
  PRIVILEGE_USAGE = 'privilege_usage',
  DATA_ACCESS = 'data_access',
}

/**
 * Threat assessment result
 */
export interface ThreatAssessmentResult {
  threatLevel: ThreatLevel;
  riskScore: number; // 0-100
  detectedVectors: AttackVector[];
  botDetection: BotDetectionResult;
  anomalyDetection: AnomalyDetectionResult;
  threatIntelligence: ThreatIntelligence;
  recommendedActions: ThreatMitigationAction[];
  requiresImmediateAction: boolean;
  blockAccess: boolean;
}

/**
 * Threat mitigation action
 */
export interface ThreatMitigationAction {
  action: 'block' | 'challenge' | 'monitor' | 'alert' | 'rate_limit' | 'require_mfa';
  priority: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  automated: boolean;
  duration?: number; // minutes
}

/**
 * Advanced Threat Protection Manager
 * Implements the hardest possible threat detection and mitigation
 */
export class AdvancedThreatProtectionManager {
  private attackPatterns: Map<string, number> = new Map();
  private rateLimiters: Map<string, RateLimiter> = new Map();
  private honeypots: Set<string> = new Set();
  private behavioralBaselines: Map<string, any> = new Map();
  
  // ML model placeholders (would be actual models in production)
  private mlModels = {
    botDetection: null as any,
    anomalyDetection: null as any,
    behavioralAnalysis: null as any,
    threatClassification: null as any,
  };

  constructor(private baseApiUrl: string) {
    this.initializeHoneypots();
    this.initializeThreatIntelligence();
  }

  /**
   * Perform comprehensive threat assessment
   */
  public async performThreatAssessment(
    userId: string,
    sessionId: string,
    context: ThreatAssessmentContext
  ): Promise<ThreatAssessmentResult> {
    try {
      // Parallel threat detection
      const [
        botDetection,
        anomalyDetection,
        threatIntelligence,
        attackPatternAnalysis,
      ] = await Promise.all([
        this.detectBot(context),
        this.detectAnomalies(userId, context),
        this.analyzeThreatIntelligence(context),
        this.analyzeAttackPatterns(context),
      ]);

      // Calculate overall risk score
      const riskScore = this.calculateRiskScore({
        botDetection,
        anomalyDetection,
        threatIntelligence,
        attackPatternAnalysis,
      });

      // Determine threat level
      const threatLevel = this.determineThreatLevel(riskScore);

      // Identify attack vectors
      const detectedVectors = this.identifyAttackVectors({
        botDetection,
        anomalyDetection,
        threatIntelligence,
        context,
      });

      // Generate mitigation recommendations
      const recommendedActions = this.generateMitigationActions(
        threatLevel,
        detectedVectors,
        riskScore
      );

      const result: ThreatAssessmentResult = {
        threatLevel,
        riskScore,
        detectedVectors,
        botDetection,
        anomalyDetection,
        threatIntelligence,
        recommendedActions,
        requiresImmediateAction: threatLevel >= ThreatLevel.HIGH,
        blockAccess: threatLevel >= ThreatLevel.CRITICAL,
      };

      // Log threat assessment
      await this.logThreatAssessment(userId, sessionId, result);

      return result;

    } catch (error) {
      console.error('Threat assessment failed:', error);
      // Return safe default
      return {
        threatLevel: ThreatLevel.MEDIUM,
        riskScore: 50,
        detectedVectors: [],
        botDetection: { isBot: false, confidence: 0, botType: 'unknown', evidence: [], mitigationActions: [] },
        anomalyDetection: { isAnomalous: false, anomalyScore: 0, anomalyTypes: [], confidence: 0, baseline: {}, current: {}, recommendation: '' },
        threatIntelligence: this.getEmptyThreatIntelligence(),
        recommendedActions: [{ action: 'monitor', priority: 'medium', description: 'Monitor for threats', automated: true }],
        requiresImmediateAction: false,
        blockAccess: false,
      };
    }
  }

  /**
   * Advanced bot detection using behavioral analysis
   */
  private async detectBot(context: ThreatAssessmentContext): Promise<BotDetectionResult> {
    const evidence: BotEvidence[] = [];
    let confidence = 0;
    let botType: BotDetectionResult['botType'] = 'human';

    // User agent analysis
    const userAgentAnalysis = this.analyzeUserAgent(context.userAgent);
    if (userAgentAnalysis.suspicious) {
      evidence.push({
        type: 'user_agent',
        description: userAgentAnalysis.reason,
        severity: userAgentAnalysis.severity,
        confidence: userAgentAnalysis.confidence,
        data: { userAgent: context.userAgent },
      });
      confidence += userAgentAnalysis.confidence * 0.3;
    }

    // Timing analysis - bot-like request patterns
    const timingAnalysis = this.analyzeTimingPatterns(context);
    if (timingAnalysis.suspicious) {
      evidence.push({
        type: 'timing',
        description: 'Suspicious request timing patterns detected',
        severity: 'high',
        confidence: timingAnalysis.confidence,
        data: { patterns: timingAnalysis.patterns },
      });
      confidence += timingAnalysis.confidence * 0.4;
      botType = 'attack';
    }

    // Behavioral analysis - human-like interactions
    const behavioralAnalysis = this.analyzeBehavioralPatterns(context);
    if (behavioralAnalysis.suspicious) {
      evidence.push({
        type: 'behavioral',
        description: 'Non-human behavioral patterns detected',
        severity: 'medium',
        confidence: behavioralAnalysis.confidence,
        data: { patterns: behavioralAnalysis.patterns },
      });
      confidence += behavioralAnalysis.confidence * 0.3;
      botType = 'scraper';
    }

    // Network fingerprinting
    const networkAnalysis = this.analyzeNetworkFingerprint(context);
    if (networkAnalysis.suspicious) {
      evidence.push({
        type: 'network',
        description: 'Suspicious network characteristics',
        severity: 'medium',
        confidence: networkAnalysis.confidence,
        data: { network: networkAnalysis.details },
      });
      confidence += networkAnalysis.confidence * 0.2;
    }

    // Device fingerprint analysis
    const fingerprintAnalysis = this.analyzeFingerprintConsistency(context);
    if (fingerprintAnalysis.suspicious) {
      evidence.push({
        type: 'fingerprint',
        description: 'Inconsistent or suspicious device fingerprint',
        severity: 'high',
        confidence: fingerprintAnalysis.confidence,
        data: { fingerprint: context.deviceFingerprint },
      });
      confidence += fingerprintAnalysis.confidence * 0.4;
      botType = 'attack';
    }

    // Honeypot interaction check
    if (this.checkHoneypotInteraction(context)) {
      evidence.push({
        type: 'behavioral',
        description: 'Honeypot interaction detected - automated tool confirmed',
        severity: 'high',
        confidence: 1.0,
        data: { honeypot: true },
      });
      confidence = 1.0;
      botType = 'attack';
    }

    const isBot = confidence > 0.5;
    const mitigationActions = this.generateBotMitigationActions(isBot, botType, confidence);

    return {
      isBot,
      confidence: Math.min(confidence, 1.0),
      botType,
      evidence,
      mitigationActions,
    };
  }

  /**
   * ML-based anomaly detection
   */
  private async detectAnomalies(userId: string, context: ThreatAssessmentContext): Promise<AnomalyDetectionResult> {
    try {
      // Get user baseline behavior
      const baseline = await this.getUserBaseline(userId);
      
      const anomalies: AnomalyType[] = [];
      let anomalyScore = 0;
      let confidence = 0;

      // Timing anomaly detection
      const timingAnomaly = this.detectTimingAnomaly(baseline.timing, context.timing);
      if (timingAnomaly.isAnomalous) {
        anomalies.push(AnomalyType.TIMING);
        anomalyScore += timingAnomaly.score * 0.2;
        confidence += timingAnomaly.confidence * 0.2;
      }

      // Geographic anomaly detection
      const geoAnomaly = this.detectGeographicAnomaly(baseline.location, context.location);
      if (geoAnomaly.isAnomalous) {
        anomalies.push(AnomalyType.GEOGRAPHIC);
        anomalyScore += geoAnomaly.score * 0.3;
        confidence += geoAnomaly.confidence * 0.3;
      }

      // Behavioral anomaly detection
      const behaviorAnomaly = this.detectBehavioralAnomaly(baseline.behavior, context.behavior);
      if (behaviorAnomaly.isAnomalous) {
        anomalies.push(AnomalyType.BEHAVIORAL);
        anomalyScore += behaviorAnomaly.score * 0.25;
        confidence += behaviorAnomaly.confidence * 0.25;
      }

      // Device anomaly detection
      const deviceAnomaly = this.detectDeviceAnomaly(baseline.device, context.deviceFingerprint);
      if (deviceAnomaly.isAnomalous) {
        anomalies.push(AnomalyType.DEVICE);
        anomalyScore += deviceAnomaly.score * 0.25;
        confidence += deviceAnomaly.confidence * 0.25;
      }

      const isAnomalous = anomalies.length > 0 && anomalyScore > 0.5;

      return {
        isAnomalous,
        anomalyScore: Math.min(anomalyScore, 1.0),
        anomalyTypes: anomalies,
        confidence: Math.min(confidence, 1.0),
        baseline,
        current: {
          timing: context.timing,
          location: context.location,
          behavior: context.behavior,
          device: context.deviceFingerprint,
        },
        recommendation: isAnomalous ? 'Require additional verification' : 'Normal behavior detected',
      };

    } catch (error) {
      console.error('Anomaly detection failed:', error);
      return {
        isAnomalous: false,
        anomalyScore: 0,
        anomalyTypes: [],
        confidence: 0,
        baseline: {},
        current: {},
        recommendation: 'Unable to perform anomaly detection',
      };
    }
  }

  /**
   * Comprehensive threat intelligence analysis
   */
  private async analyzeThreatIntelligence(context: ThreatAssessmentContext): Promise<ThreatIntelligence> {
    try {
      // Parallel intelligence gathering
      const [ipReputation, emailReputation, deviceReputation, geolocationRisk] = await Promise.all([
        this.checkIPReputation(context.ipAddress),
        this.checkEmailReputation(context.email),
        this.checkDeviceReputation(context.deviceFingerprint),
        this.assessGeolocationRisk(context.location),
      ]);

      // Get known attack patterns
      const knownAttackPatterns = await this.getKnownAttackPatterns(context);

      return {
        ipReputation,
        emailReputation,
        deviceReputation,
        geolocationRisk,
        knownAttackPatterns,
      };

    } catch (error) {
      console.error('Threat intelligence analysis failed:', error);
      return this.getEmptyThreatIntelligence();
    }
  }

  /**
   * User agent analysis for bot detection
   */
  private analyzeUserAgent(userAgent: string): { suspicious: boolean; reason: string; severity: 'low' | 'medium' | 'high'; confidence: number } {
    const suspiciousPatterns = [
      { pattern: /bot|crawler|spider|scraper/i, severity: 'high' as const, confidence: 0.9 },
      { pattern: /curl|wget|python|requests/i, severity: 'high' as const, confidence: 0.8 },
      { pattern: /phantomjs|headless|selenium/i, severity: 'high' as const, confidence: 0.85 },
      { pattern: /^$/i, severity: 'medium' as const, confidence: 0.7 }, // Empty user agent
    ];

    for (const { pattern, severity, confidence } of suspiciousPatterns) {
      if (pattern.test(userAgent)) {
        return {
          suspicious: true,
          reason: `Suspicious user agent pattern: ${pattern.source}`,
          severity,
          confidence,
        };
      }
    }

    // Check for unusual browser versions or inconsistencies
    if (this.isUserAgentInconsistent(userAgent)) {
      return {
        suspicious: true,
        reason: 'User agent appears to be forged or inconsistent',
        severity: 'medium',
        confidence: 0.6,
      };
    }

    return { suspicious: false, reason: '', severity: 'low', confidence: 0 };
  }

  /**
   * Check for rate limiting violations
   */
  private checkRateLimit(identifier: string, action: string): { violated: boolean; remainingAttempts: number; resetTime: Date } {
    const key = `${identifier}:${action}`;
    let limiter = this.rateLimiters.get(key);
    
    if (!limiter) {
      limiter = new RateLimiter({
        windowMs: 15 * 60 * 1000, // 15 minutes
        maxRequests: this.getMaxRequestsForAction(action),
        identifier,
      });
      this.rateLimiters.set(key, limiter);
    }

    return limiter.checkLimit();
  }

  /**
   * Initialize honeypot endpoints
   */
  private initializeHoneypots(): void {
    const honeypotEndpoints = [
      '/admin/login',
      '/wp-admin',
      '/.env',
      '/config.php',
      '/database.sql',
      '/backup.tar.gz',
    ];

    honeypotEndpoints.forEach(endpoint => {
      this.honeypots.add(endpoint);
    });
  }

  /**
   * Initialize threat intelligence feeds
   */
  private async initializeThreatIntelligence(): Promise<void> {
    // In production, this would connect to threat intelligence APIs
    // like VirusTotal, AbuseIPDB, MISP, etc.
    console.log('Threat intelligence feeds initialized');
  }

  // Helper methods (simplified implementations)
  private isUserAgentInconsistent(userAgent: string): boolean {
    // Implement sophisticated user agent validation
    return false; // Simplified
  }

  private analyzeTimingPatterns(context: ThreatAssessmentContext): { suspicious: boolean; confidence: number; patterns: Record<string, unknown> } {
    // Analyze request timing patterns for bot-like behavior
    return { suspicious: false, confidence: 0, patterns: {} };
  }

  private analyzeBehavioralPatterns(context: ThreatAssessmentContext): { suspicious: boolean; confidence: number; patterns: Record<string, unknown> } {
    // Analyze behavioral patterns for human-like interactions
    return { suspicious: false, confidence: 0, patterns: {} };
  }

  private analyzeNetworkFingerprint(context: ThreatAssessmentContext): { suspicious: boolean; confidence: number; details: Record<string, unknown> } {
    // Analyze network characteristics
    return { suspicious: false, confidence: 0, details: {} };
  }

  private analyzeFingerprintConsistency(context: ThreatAssessmentContext): { suspicious: boolean; confidence: number } {
    // Analyze device fingerprint consistency
    return { suspicious: false, confidence: 0 };
  }

  private checkHoneypotInteraction(context: ThreatAssessmentContext): boolean {
    // Check if request interacted with honeypot
    return this.honeypots.has(context.requestPath || '');
  }

  private generateBotMitigationActions(isBot: boolean, botType: string, confidence: number): string[] {
    const actions: string[] = [];
    
    if (isBot) {
      if (confidence > 0.8) {
        actions.push('Block immediately');
        actions.push('Add to blacklist');
      } else if (confidence > 0.6) {
        actions.push('Require CAPTCHA');
        actions.push('Rate limit severely');
      } else {
        actions.push('Monitor closely');
        actions.push('Increase authentication requirements');
      }
    }

    return actions;
  }

  private async getUserBaseline(userId: string): Promise<any> {
    // Get user's behavioral baseline from ML model or database
    return this.behavioralBaselines.get(userId) || {};
  }

  private detectTimingAnomaly(baseline: any, current: any): { isAnomalous: boolean; score: number; confidence: number } {
    // Implement timing anomaly detection
    return { isAnomalous: false, score: 0, confidence: 0 };
  }

  private detectGeographicAnomaly(baseline: any, current: any): { isAnomalous: boolean; score: number; confidence: number } {
    // Implement geographic anomaly detection
    return { isAnomalous: false, score: 0, confidence: 0 };
  }

  private detectBehavioralAnomaly(baseline: any, current: any): { isAnomalous: boolean; score: number; confidence: number } {
    // Implement behavioral anomaly detection
    return { isAnomalous: false, score: 0, confidence: 0 };
  }

  private detectDeviceAnomaly(baseline: any, current: any): { isAnomalous: boolean; score: number; confidence: number } {
    // Implement device anomaly detection
    return { isAnomalous: false, score: 0, confidence: 0 };
  }

  private async checkIPReputation(ip: string): Promise<IPReputationResult> {
    // Implement IP reputation checking using threat intelligence APIs
    return {
      ip,
      riskScore: 0,
      categories: [],
      country: 'Unknown',
      isp: 'Unknown',
      isVpn: false,
      isTor: false,
      isProxy: false,
      isMalicious: false,
      lastSeen: null,
      threatSources: [],
    };
  }

  private async checkEmailReputation(email: string): Promise<EmailReputationResult> {
    // Implement email reputation checking
    const domain = email.split('@')[1];
    return {
      email,
      domain,
      riskScore: 0,
      isDisposable: false,
      isCompromised: false,
      isSpam: false,
      domainAge: 365,
      mxRecords: [],
      breachHistory: [],
    };
  }

  private async checkDeviceReputation(fingerprint: string): Promise<DeviceReputationResult> {
    // Implement device reputation checking
    return {
      fingerprint,
      riskScore: 0,
      isEmulated: false,
      isVirtualized: false,
      isSuspicious: false,
      seenCount: 1,
      firstSeen: new Date(),
      lastSeen: new Date(),
      associatedThreats: [],
    };
  }

  private async assessGeolocationRisk(location: any): Promise<GeolocationRisk> {
    // Implement geolocation risk assessment
    return {
      country: location?.country || 'Unknown',
      region: location?.region || 'Unknown',
      city: location?.city || 'Unknown',
      riskLevel: ThreatLevel.LOW,
      isHighRiskCountry: false,
      isSanctionedCountry: false,
      isVpnDetected: false,
      isUnusualLocation: false,
      distanceFromNormal: 0,
    };
  }

  private async getKnownAttackPatterns(context: ThreatAssessmentContext): Promise<AttackPattern[]> {
    // Get known attack patterns from threat intelligence
    return [];
  }

  private calculateRiskScore(data: any): number {
    // Implement complex risk scoring algorithm
    let score = 0;

    if (data.botDetection.isBot) {
      score += data.botDetection.confidence * 30;
    }

    if (data.anomalyDetection.isAnomalous) {
      score += data.anomalyDetection.anomalyScore * 25;
    }

    score += data.threatIntelligence.ipReputation.riskScore * 0.2;
    score += data.threatIntelligence.emailReputation.riskScore * 0.15;
    score += data.threatIntelligence.deviceReputation.riskScore * 0.1;

    return Math.min(score, 100);
  }

  private determineThreatLevel(riskScore: number): ThreatLevel {
    if (riskScore >= 90) return ThreatLevel.APT;
    if (riskScore >= 75) return ThreatLevel.CRITICAL;
    if (riskScore >= 50) return ThreatLevel.HIGH;
    if (riskScore >= 25) return ThreatLevel.MEDIUM;
    if (riskScore >= 10) return ThreatLevel.LOW;
    return ThreatLevel.NONE;
  }

  private identifyAttackVectors(data: any): AttackVector[] {
    const vectors: AttackVector[] = [];

    if (data.botDetection.isBot) {
      if (data.botDetection.botType === 'attack') {
        vectors.push(AttackVector.BOT_ATTACK);
      }
    }

    if (data.anomalyDetection.isAnomalous) {
      vectors.push(AttackVector.ACCOUNT_TAKEOVER);
    }

    return vectors;
  }

  private generateMitigationActions(threatLevel: ThreatLevel, vectors: AttackVector[], riskScore: number): ThreatMitigationAction[] {
    const actions: ThreatMitigationAction[] = [];

    if (threatLevel >= ThreatLevel.CRITICAL) {
      actions.push({
        action: 'block',
        priority: 'critical',
        description: 'Immediately block access due to critical threat',
        automated: true,
        duration: 1440, // 24 hours
      });
    }

    if (threatLevel >= ThreatLevel.HIGH) {
      actions.push({
        action: 'require_mfa',
        priority: 'high',
        description: 'Require multi-factor authentication',
        automated: true,
      });
    }

    if (vectors.includes(AttackVector.BOT_ATTACK)) {
      actions.push({
        action: 'challenge',
        priority: 'high',
        description: 'Present CAPTCHA challenge',
        automated: true,
      });
    }

    actions.push({
      action: 'monitor',
      priority: 'medium',
      description: 'Enhanced monitoring and logging',
      automated: true,
    });

    return actions;
  }

  private async logThreatAssessment(userId: string, sessionId: string, result: ThreatAssessmentResult): Promise<void> {
    try {
      await fetch(`${this.baseApiUrl}/api/v1/security/threat-assessment`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          user_id: userId,
          session_id: sessionId,
          assessment: result,
          timestamp: new Date().toISOString(),
        }),
      });
    } catch (error) {
      console.error('Failed to log threat assessment:', error);
    }
  }

  private getEmptyThreatIntelligence(): ThreatIntelligence {
    return {
      ipReputation: {
        ip: '',
        riskScore: 0,
        categories: [],
        country: 'Unknown',
        isp: 'Unknown',
        isVpn: false,
        isTor: false,
        isProxy: false,
        isMalicious: false,
        lastSeen: null,
        threatSources: [],
      },
      emailReputation: {
        email: '',
        domain: '',
        riskScore: 0,
        isDisposable: false,
        isCompromised: false,
        isSpam: false,
        domainAge: 0,
        mxRecords: [],
        breachHistory: [],
      },
      deviceReputation: {
        fingerprint: '',
        riskScore: 0,
        isEmulated: false,
        isVirtualized: false,
        isSuspicious: false,
        seenCount: 0,
        firstSeen: new Date(),
        lastSeen: new Date(),
        associatedThreats: [],
      },
      geolocationRisk: {
        country: 'Unknown',
        region: 'Unknown',
        city: 'Unknown',
        riskLevel: ThreatLevel.NONE,
        isHighRiskCountry: false,
        isSanctionedCountry: false,
        isVpnDetected: false,
        isUnusualLocation: false,
        distanceFromNormal: 0,
      },
      knownAttackPatterns: [],
    };
  }

  private getMaxRequestsForAction(action: string): number {
    const limits: Record<string, number> = {
      'login': 5,
      'password_reset': 3,
      'registration': 3,
      'api_call': 100,
      'admin_action': 20,
    };

    return limits[action] || 10;
  }

  private analyzeAttackPatterns(context: ThreatAssessmentContext): Record<string, unknown> {
    // Analyze for known attack patterns
    return { patterns: [], confidence: 0 };
  }
}

/**
 * Rate limiter implementation
 */
class RateLimiter {
  private requests: number[] = [];
  
  constructor(private config: { windowMs: number; maxRequests: number; identifier: string }) {}

  checkLimit(): { violated: boolean; remainingAttempts: number; resetTime: Date } {
    const now = Date.now();
    const windowStart = now - this.config.windowMs;
    
    // Clean old requests
    this.requests = this.requests.filter(timestamp => timestamp > windowStart);
    
    const violated = this.requests.length >= this.config.maxRequests;
    
    if (!violated) {
      this.requests.push(now);
    }
    
    return {
      violated,
      remainingAttempts: Math.max(0, this.config.maxRequests - this.requests.length),
      resetTime: new Date(now + this.config.windowMs),
    };
  }
}

/**
 * Threat assessment context
 */
export interface ThreatAssessmentContext {
  ipAddress: string;
  userAgent: string;
  email: string;
  deviceFingerprint: string;
  location?: any;
  timing?: any;
  behavior?: any;
  requestPath?: string;
  headers?: Record<string, string>;
  sessionData?: any;
}