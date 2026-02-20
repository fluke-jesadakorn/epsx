'use client';

import { useState, useEffect, useCallback } from 'react';
import { useRouter, useParams } from 'next/navigation';
import type { ChatConversation, ChatMessage } from '@/shared/api/chat';
import {
  getConversationAction,
  getMessagesAction,
  sendMessageAction,
  updateConversationStatusAction,
  markConversationReadAction
} from '@/app/actions/chat';
import { ChatHeader } from '@/components/chat/chat-header';
import { ChatMessageList } from '@/components/chat/chat-message-list';
import { ChatInput } from '@/components/chat/chat-input';
import { useSharedAuth } from '@/shared/components/auth';
import { MessageCircle, Loader2, Shield } from 'lucide-react';
import { useChatSSE } from '@/shared/hooks/use-chat-sse';
import type { ChatSSEEvent } from '@/shared/hooks/use-chat-sse';

export default function ChatConversationPage() {
  const router = useRouter();
  const params = useParams();
  const id = params?.id as string;
  const { isAuthenticated, getWalletAddress } = useSharedAuth();
  const [convo, setConvo] = useState<ChatConversation | null>(null);
  const [msgs, setMsgs] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      if (!id) {
        return;
      }
      const [convoData, msgsData] = await Promise.all([
        getConversationAction(id),
        getMessagesAction(id)
      ]);
      setConvo(convoData);
      setMsgs(msgsData);
      void markConversationReadAction(id);
      setLoading(false);
    };
    void load();
  }, [id]);

  const handleSendMsg = useCallback(async (content: string) => {
    if (!id) {
      return;
    }
    const msg = await sendMessageAction(id, content);
    if (msg) {
      setMsgs((prev) => [...prev, msg]);
    }
  }, [id]);

  const handleResolve = useCallback(async () => {
    if (!id) {
      return;
    }
    const updated = await updateConversationStatusAction(id, 'resolved');
    if (updated) {
      setConvo(updated);
    }
  }, [id]);

  const handleBack = useCallback(() => {
    router.push('/chat');
  }, [router]);

  // SSE: real-time messages & status updates
  const handleSSE = useCallback((evt: ChatSSEEvent) => {
    if (evt.type === 'new_message' && evt.message && evt.conversation_id === id) {
      setMsgs((prev) => prev.some((m) => m.id === evt.message?.id) ? prev : [...prev, evt.message as ChatMessage]);
    }
    if (evt.type === 'status_changed' && evt.conversation && evt.conversation_id === id) {
      setConvo(evt.conversation);
    }
  }, [id]);

  useChatSSE({ enabled: isAuthenticated && Boolean(id), mode: 'user', onEvent: handleSSE });

  if (!isAuthenticated) {
    return (
      <div className="container mx-auto px-4 py-20 max-w-md text-center">
        <div className="w-20 h-20 rounded-3xl bg-gradient-to-br from-blue-500/20 to-indigo-500/10 flex items-center justify-center mx-auto mb-6 border border-blue-500/10">
          <Shield className="w-10 h-10 text-blue-400" />
        </div>
        <h1 className="text-2xl font-bold mb-2">Support Chat</h1>
        <p className="text-muted-foreground text-sm">Please sign in to access support chat.</p>
      </div>
    );
  }

  if (loading || !convo) {
    return (
      <div className="container mx-auto px-4 py-20 max-w-md text-center">
        <Loader2 className="w-8 h-8 text-blue-400 animate-spin mx-auto mb-4" />
        <p className="text-sm text-muted-foreground">Loading conversation...</p>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8 max-w-2xl">
      <div className="bg-white dark:bg-slate-900/30 border border-slate-200 dark:border-white/5 rounded-3xl overflow-hidden flex flex-col shadow-sm" style={{ height: '720px' }}>
        <ChatHeader
          subject={convo.subject}
          status={convo.status}
          onBack={handleBack}
          onResolve={handleResolve}
          showResolve={true}
        />
        <ChatMessageList msgs={msgs} userAddr={getWalletAddress() ?? undefined} />
        <ChatInput onSend={handleSendMsg} disabled={convo.status === 'closed'} />
      </div>
    </div>
  );
}
