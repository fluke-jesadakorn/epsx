/**
 * Unified Client-Side Fetch Utility
 * Direct fetch to backend with bearer token authentication
 */

'use client'

import { env } from '@/config/env'

const BACKEND_URL = env.BACKEND_URL

function getToken(): string | null {
  if (typeof document === 'undefined') return null

  const cookies = document.cookie.split(';')
  const tokenCookie = cookies.find(c => c.trim().startsWith('access_token='))
  return tokenCookie?.split('=')[1] || null
}

export async function apiFetch<T = any>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const token = getToken()

  const res = await fetch(`${BACKEND_URL}${endpoint}`, {
    ...options,
    headers: {
      ...(token && { 'Authorization': `Bearer ${token}` }),
      'Content-Type': 'application/json',
      ...options?.headers
    },
    credentials: 'include'
  })

  if (!res.ok) {
    const err = await res.text().catch(() => 'Request failed')
    throw new Error(`HTTP ${res.status}: ${err}`)
  }

  return res.json()
}

export async function apiGet<T = any>(endpoint: string): Promise<T> {
  return apiFetch<T>(endpoint, { method: 'GET' })
}

export async function apiPost<T = any>(endpoint: string, body?: any): Promise<T> {
  return apiFetch<T>(endpoint, {
    method: 'POST',
    body: body ? JSON.stringify(body) : undefined
  })
}

export async function apiPut<T = any>(endpoint: string, body?: any): Promise<T> {
  return apiFetch<T>(endpoint, {
    method: 'PUT',
    body: body ? JSON.stringify(body) : undefined
  })
}

export async function apiDelete<T = any>(endpoint: string): Promise<T> {
  return apiFetch<T>(endpoint, { method: 'DELETE' })
}
