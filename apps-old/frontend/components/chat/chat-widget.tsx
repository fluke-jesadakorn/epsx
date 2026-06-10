'use client';

import { useState, useEffect, useCallback } from 'react';
import { usePathname } from 'next/navigation';
import { ChatBubble } from './chat-bubble';
import { ChatPanel } from './chat-panel';
import { useSharedAuth } from '@/shared/components/auth';
import { getUnreadCountAction } from '@/app/actions/chat';
import { useChatSSE } from '@/shared/hooks/use-chat-sse';
import type { ChatSSEEvent } from '@/shared/hooks/use-chat-sse';

const PANEL_STATE_KEY = 'epsx_chat_panel_open';

export function ChatWidget() {
  const { isAuthenticated, getWalletAddress } = useSharedAuth();
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);
  const [unread, setUnread] = useState(0);

  useEffect(() => {
    const stored = sessionStorage.getItem(PANEL_STATE_KEY);
    if (stored === 'true') {
      setIsOpen(true);
    }
  }, []);

  useEffect(() => {
    sessionStorage.setItem(PANEL_STATE_KEY, String(isOpen));
  }, [isOpen]);

  // Load initial unread count
  useEffect(() => {
    if (isAuthenticated) {
      void getUnreadCountAction().then(setUnread);
    }
  }, [isAuthenticated]);

  // SSE: real-time unread updates (replaces 30s polling)
  const handleSSE = useCallback((evt: ChatSSEEvent) => {
    if (evt.type === 'new_message') {
      setUnread((prev) => prev + 1);
    }
  }, []);

  useChatSSE({ enabled: isAuthenticated, mode: 'user', onEvent: handleSSE });

  const handleToggle = useCallback(() => {
    setIsOpen((prev) => !prev);
  }, []);

  const handleClose = useCallback(() => {
    setIsOpen(false);
  }, []);

  if (!isAuthenticated || pathname === '/chat') {
    return null;
  }

  return (
    <>
      <ChatPanel isOpen={isOpen} onClose={handleClose} walletAddr={getWalletAddress() ?? undefined} />
      <div className="fixed bottom-6 right-6 z-50">
        <ChatBubble unread={unread} onClick={handleToggle} isOpen={isOpen} />
      </div>
    </>
  );
}
