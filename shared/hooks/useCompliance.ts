/**
 * COMPLIANCE HOOKS
 *
 * React hooks for KYC, risk assessment, and compliance monitoring.
 * Provides automatic data fetching, caching, and mutation handling.
 */

'use client';

import { useCallback, useEffect, useState } from 'react';
import type {
  ComplianceMetrics,
  KYCStatus,
  RiskAssessment,
  SuspiciousActivity
} from '../api/compliance';
import { useApiClient } from './useApiClient';

// ============================================================================
// TYPES
// ============================================================================

interface UseDataResult<T> {
  data: T | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

interface UseMutationResult<TData, TVariables> {
  mutate: (variables: TVariables) => Promise<TData>;
  data: TData | null;
  loading: boolean;
  error: Error | null;
  reset: () => void;
}

// ============================================================================
// KYC HOOKS
// ============================================================================

/**
 * Get KYC statuses
 *
 * @example
 * const { data: kycStatuses, loading } = useKYCStatuses({ status: 'pending' });
 */
export function useKYCStatuses(filters?: { status?: string; limit?: number }): UseDataResult<KYCStatus[]> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<KYCStatus[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchKYCStatuses = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.getKYCStatuses(filters);
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch KYC statuses');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [compliance, JSON.stringify(filters)]);

  useEffect(() => {
    fetchKYCStatuses();
  }, [fetchKYCStatuses]);

  return { data, loading, error, refetch: fetchKYCStatuses };
}

/**
 * Approve KYC
 *
 * @example
 * const { mutate: approveKYC } = useApproveKYC();
 * await approveKYC({
 *   wallet_address: '0x123...',
 *   verification_level: 'advanced',
 *   notes: 'Verified'
 * });
 */
export function useApproveKYC(): UseMutationResult<
  { approved: boolean },
  { wallet_address: string; verification_level?: string; notes?: string }
> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<{ approved: boolean } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ wallet_address, verification_level, notes }: { wallet_address: string; verification_level?: string; notes?: string }) => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.approveKYC(wallet_address, verification_level, notes);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to approve KYC');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [compliance]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

/**
 * Reject KYC
 *
 * @example
 * const { mutate: rejectKYC } = useRejectKYC();
 * await rejectKYC({ wallet_address: '0x123...', reason: 'Invalid documents' });
 */
export function useRejectKYC(): UseMutationResult<
  { rejected: boolean },
  { wallet_address: string; reason: string }
> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<{ rejected: boolean } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ wallet_address, reason }: { wallet_address: string; reason: string }) => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.rejectKYC(wallet_address, reason);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to reject KYC');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [compliance]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// RISK ASSESSMENT HOOKS
// ============================================================================

/**
 * Get risk assessments
 *
 * @example
 * const { data: assessments } = useRiskAssessments({ risk_level: 'high' });
 */
export function useRiskAssessments(filters?: {
  risk_level?: string;
  status?: string;
  limit?: number;
}): UseDataResult<RiskAssessment[]> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<RiskAssessment[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchAssessments = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.getRiskAssessments(filters);
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch risk assessments');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [compliance, JSON.stringify(filters)]);

  useEffect(() => {
    fetchAssessments();
  }, [fetchAssessments]);

  return { data, loading, error, refetch: fetchAssessments };
}

/**
 * Update risk assessment
 *
 * @example
 * const { mutate: updateAssessment } = useUpdateRiskAssessment();
 * await updateAssessment({
 *   assessment_id: '123',
 *   status: 'resolved',
 *   notes: 'Issue resolved'
 * });
 */
export function useUpdateRiskAssessment(): UseMutationResult<
  RiskAssessment,
  { assessment_id: string } & Partial<RiskAssessment>
> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<RiskAssessment | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ assessment_id, ...updates }: { assessment_id: string } & Partial<RiskAssessment>) => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.updateRiskAssessment(assessment_id, updates);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to update risk assessment');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [compliance]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// SUSPICIOUS ACTIVITY HOOKS
// ============================================================================

/**
 * Get suspicious activities
 *
 * @example
 * const { data: activities } = useSuspiciousActivities({ status: 'new' });
 */
export function useSuspiciousActivities(filters?: {
  status?: string;
  severity?: string;
  limit?: number;
}): UseDataResult<SuspiciousActivity[]> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<SuspiciousActivity[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchActivities = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.getSuspiciousActivities(filters);
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch suspicious activities');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [compliance, JSON.stringify(filters)]);

  useEffect(() => {
    fetchActivities();
  }, [fetchActivities]);

  return { data, loading, error, refetch: fetchActivities };
}

/**
 * Flag user for suspicious activity
 *
 * @example
 * const { mutate: flagUser } = useFlagUser();
 * await flagUser({
 *   wallet_address: '0x123...',
 *   reason: 'Multiple failed transactions',
 *   severity: 'high'
 * });
 */
export function useFlagUser(): UseMutationResult<
  { flagged: boolean },
  { wallet_address: string; reason: string; severity?: string }
> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<{ flagged: boolean } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ wallet_address, reason, severity }: { wallet_address: string; reason: string; severity?: string }) => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.flagUser(wallet_address, reason, severity);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to flag user');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [compliance]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

/**
 * Block user
 *
 * @example
 * const { mutate: blockUser } = useBlockUser();
 * await blockUser({
 *   wallet_address: '0x123...',
 *   reason: 'Security violation',
 *   duration: 86400
 * });
 */
export function useBlockUser(): UseMutationResult<
  { blocked: boolean },
  { wallet_address: string; reason: string; duration?: number }
> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<{ blocked: boolean } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ wallet_address, reason, duration }: { wallet_address: string; reason: string; duration?: number }) => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.blockUser(wallet_address, reason, duration);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to block user');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [compliance]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// METRICS HOOKS
// ============================================================================

/**
 * Get compliance metrics
 *
 * @example
 * const { data: metrics, loading } = useComplianceMetrics();
 */
export function useComplianceMetrics(): UseDataResult<ComplianceMetrics> {
  const { compliance } = useApiClient();
  const [data, setData] = useState<ComplianceMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchMetrics = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await compliance.getMetrics();
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch metrics');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [compliance]);

  useEffect(() => {
    fetchMetrics();
  }, [fetchMetrics]);

  return { data, loading, error, refetch: fetchMetrics };
}

// ============================================================================
// EXPORTS
// ============================================================================

export default useKYCStatuses;
