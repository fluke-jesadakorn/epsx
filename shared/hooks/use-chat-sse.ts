'use client';

import { useEffect, useRef, useCallback } from 'react';
import type { ChatMessage, ChatConversation } from '../api/chat';
import { COOKIES } from '../auth/cookies';
import { API_ROUTES } from '../config/route-constants';

// ============================================================================
// TYPES
// ============================================================================

export interface ChatSSEEvent {
  type: 'new_message' | 'new_conversation' | 'status_changed' | 'agent_assigned' | 'typing_start' | 'typing_stop' | 'messages_read';
  conversation_id: string;
  message?: ChatMessage;
  status?: string;
  assigned_agent?: string;
  conversation?: ChatConversation;
  wallet_address?: string;
  subject?: string;
  sender?: 'user' | 'agent';
  reader?: 'user' | 'agent';
}

interface UseChatSSEOpts {
  enabled: boolean;
  mode: 'user' | 'admin';
  onEvent?: (evt: ChatSSEEvent) => void;
}

// ============================================================================
// TOKEN EXTRACTION (same pattern as notifications SSE)
// ============================================================================

function getTokenFromCookies(): string | null {
  if (typeof document === 'undefined') return null;
  try {
    const cookies: Record<string, string> = {};
    for (const c of document.cookie.split(';')) {
      const [k, v] = c.trim().split('=');
      if (k && v) cookies[k] = v;
    }
    const raw = cookies[COOKIES.user];
    if (!raw) return null;
    const user = JSON.parse(decodeURIComponent(raw)) as { access?: string };
    if (user.access && user.access.startsWith('eyJ')) return user.access;
    // Fallback: any JWT in cookies
    for (const v of Object.values(cookies)) {
      if (v.length > 50 && v.startsWith('eyJ')) return v;
    }
  } catch { /* ignore */ }
  return null;
}

function getBaseUrl(): string {
  if (typeof window !== 'undefined') {
    try {
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const { getBackendUrl } = require('../utils/url-resolver') as { getBackendUrl: (ctx: string) => string };
      return getBackendUrl('client');
    } catch { /* ignore */ }
  }
  return 'http://127.0.0.1:8080';
}

// ============================================================================
// HOOK
// ============================================================================

export function useChatSSE({ enabled, mode, onEvent }: UseChatSSEOpts) {
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;
  const esRef = useRef<EventSource | null>(null);
  const retryRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const connect = useCallback(() => {
    if (typeof EventSource === 'undefined') return;

    const token = getTokenFromCookies();
    if (!token) return;

    const base = getBaseUrl();
    const route = mode === 'admin' ? API_ROUTES.CHAT.ADMIN_STREAM : API_ROUTES.CHAT.STREAM;
    const url = `${base}${route}?token=${encodeURIComponent(token)}`;

    const es = new EventSource(url);
    esRef.current = es;

    const eventName = mode === 'admin' ? 'chat_event' : 'chat_message';

    es.addEventListener(eventName, (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data as string) as ChatSSEEvent;
        onEventRef.current?.(data);
        retryRef.current = 0;
      } catch { /* invalid payload */ }
    });

    es.onerror = () => {
      es.close();
      esRef.current = null;
      // Exponential backoff: 5s, 10s, 15s ... 30s max
      retryRef.current = Math.min(retryRef.current + 1, 6);
      const delay = Math.min(5000 * retryRef.current, 30000);
      timerRef.current = setTimeout(connect, delay);
    };
  }, [mode]);

  const disconnect = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    if (esRef.current) {
      esRef.current.close();
      esRef.current = null;
    }
    retryRef.current = 0;
  }, []);

  useEffect(() => {
    if (!enabled) {
      disconnect();
      return;
    }
    connect();
    return disconnect;
  }, [enabled, connect, disconnect]);
}
