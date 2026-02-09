/**
 * API RESPONSE UTILITIES
 *
 * Standardized helpers for extracting data from UnifiedApiResponse format.
 *
 * Backend returns: { success, data, error, meta }
 * Frontend ApiClient wraps in: { success, data: backendResponse, status }
 *
 * So the actual payload is at: response.data.data
 */

import type { ApiResponse, PaginatedResponse } from '../types/api';
import { logger } from './logger';

// ============================================================================
// DATA EXTRACTION
// ============================================================================

/**
 * Extract data from a standard API response.
 * Handles the double-wrapping: res.data contains the backend's response body,
 * and res.data.data contains the actual payload.
 */
export function extractData<T>(response: ApiResponse): T | undefined {
    // UnifiedApiClient returns the payload directly in response.data
    return response.data as T | undefined;
}

/**
 * Extract data array from API response, with validation.
 * Throws if the data is not an array.
 *
 * @param response - The API response from UnifiedApiClient
 * @param context - Context string for error messages (e.g., 'getPermissionGroups')
 */
export function extractArray<T>(response: ApiResponse, context: string): T[] {
    if (!isSuccess(response)) {
        const error = extractError(response);
        throw new Error(`${context} failed: ${error}`);
    }

    const data = extractData<T[]>(response);
    if (!Array.isArray(data)) {
        logger.error(`[${context}] Expected array but got:`, typeof data, data);
        throw new Error(`Invalid response: expected array in ${context}`);
    }
    return data;
}

/**
 * Extract data array with fallback to empty array if not found or not an array.
 * Use this when an empty result is acceptable.
 */
export function extractArrayOrEmpty<T>(response: ApiResponse): T[] {
    if (!isSuccess(response)) {
        return [];
    }
    const data = extractData<T[]>(response);
    return Array.isArray(data) ? data : [];
}

/**
 * Extract single object from API response.
 * Throws if no data is found.
 *
 * @param response - The API response from UnifiedApiClient
 * @param context - Context string for error messages
 */
export function extractObject<T>(response: ApiResponse, context: string): T {
    if (!isSuccess(response)) {
        const error = extractError(response);
        throw new Error(`${context} failed: ${error}`);
    }

    const data = extractData<T>(response);
    if (data === null || data === undefined || typeof data !== 'object') {
        logger.error(`[${context}] Expected object but got:`, typeof data, data);
        throw new Error(`Invalid response: expected object in ${context}`);
    }
    return data;
}

/**
 * Extract data with a fallback value if not present.
 */
export function extractDataOrDefault<T>(response: ApiResponse, defaultValue: T): T {
    const data = extractData<T>(response);
    return data ?? defaultValue;
}

// ============================================================================
// METADATA EXTRACTION
// ============================================================================

/**
 * Extract pagination metadata from API response.
 * Backend includes this in response.meta.pagination
 */
export function extractPagination(response: ApiResponse): PaginatedResponse<unknown>['pagination'] | undefined {
    return response.meta?.pagination as PaginatedResponse<unknown>['pagination'] | undefined;
}

/**
 * Extract the message from API response metadata.
 */
export function extractMessage(response: ApiResponse): string | undefined {
    return response.meta?.message as string | undefined;
}

// ============================================================================
// RESPONSE VALIDATION
// ============================================================================

/**
 * Check if response indicates success.
 * Checks both the client wrapper success and the backend success flag.
 */
export function isSuccess(response: ApiResponse): boolean {
    const backendSuccess = (response.data as { success?: boolean } | null)?.success;
    return response.success && backendSuccess !== false;
}

/**
 * Extract error message from API response.
 * Checks multiple locations where error info may be present.
 */
export function extractError(response: ApiResponse): string {
    const dataError = (response.data as { error?: { message?: string; reason?: string } } | null)?.error;
    return (
        response.error?.message ??
        dataError?.message ??
        dataError?.reason ??
        'Unknown error'
    );
}

/**
 * Assert the response was successful, throw otherwise.
 */
export function assertSuccess(response: ApiResponse, context: string): void {
    if (!isSuccess(response)) {
        const error = extractError(response);
        logger.error(`[${context}] Request failed:`, error);
        throw new Error(`${context} failed: ${error}`);
    }
}
