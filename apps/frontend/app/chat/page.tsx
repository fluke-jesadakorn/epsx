'use client';

import { getTopicsAction, listConversationsAction } from '@/app/actions/chat';
import { ChatInbox } from '@/components/chat/chat-inbox';
import type { ChatConversation, ChatTopic } from '@/shared/api/chat';
import { useSharedAuth } from '@/shared/components/auth';
import { Loader2, MessageCircle, Shield } from 'lucide-react';
import { useEffect, useState } from 'react';

export default function ChatPage() {
  const { isAuthenticated, getWalletAddress } = useSharedAuth();
  const [topics, setTopics] = useState<ChatTopic[]>([]);
  const [convos, setConvos] = useState<ChatConversation[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!isAuthenticated) {
      setLoading(false);
      return;
    }
    const load = async () => {
      try {
        const [t, c] = await Promise.all([getTopicsAction(), listConversationsAction()]);
        setTopics(Array.isArray(t) ? t : []);
        setConvos(Array.isArray(c) ? c : []);
      } catch { /* API unavailable */ }
      setLoading(false);
    };
    void load();
  }, [isAuthenticated]);

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

  if (loading) {
    return (
      <div className="container mx-auto px-4 py-20 max-w-md text-center">
        <Loader2 className="w-8 h-8 text-blue-400 animate-spin mx-auto mb-4" />
        <p className="text-sm text-muted-foreground">Loading conversations...</p>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-4 md:py-6">
      <div className="flex items-center gap-3 mb-4 md:mb-6">
        <div className="w-11 h-11 rounded-2xl bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center shadow-sm shadow-blue-500/20">
          <MessageCircle className="w-5 h-5 text-white" />
        </div>
        <div>
          <h1 className="text-xl font-bold tracking-tight">Support</h1>
          <p className="text-xs text-muted-foreground">Get help from our team</p>
        </div>
      </div>
      <ChatInbox
        topics={topics}
        initConvos={convos}
        userAddr={getWalletAddress() ?? undefined}
      />
    </div>
  );
}
