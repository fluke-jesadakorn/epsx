'use client';

import { getChatInboxAction } from '@/app/actions/chat';
import { ChatInbox } from '@/components/chat/chat-inbox';
import type { ChatConversation, ChatTopic } from '@/shared/api/chat';
import { AuthBanner } from '@/shared/components/auth/auth-banner';
import { useSharedAuth } from '@/shared/components/auth';
import { Headset, Loader2 } from 'lucide-react';
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
        const inbox = await getChatInboxAction();
        setTopics(inbox.topics);
        setConvos(inbox.conversations);
      } catch { /* API unavailable */ }
      setLoading(false);
    };
    void load();
  }, [isAuthenticated]);

  if (!isAuthenticated) {
    return (
      <div className="container mx-auto px-4 py-12 max-w-xl">
        <div className="flex items-center gap-4 mb-8">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] flex items-center justify-center shadow-lg shadow-[#7645d9]/25">
            <Headset className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-xl font-bold tracking-tight">Support Center</h1>
            <p className="text-xs text-muted-foreground mt-0.5">Get help from our team · Usually replies in minutes</p>
          </div>
        </div>
        <AuthBanner
          message="Sign in to access Support Chat"
          description="Connect your wallet to start a conversation with our team"
          buttonText="Sign In to Chat"
        />
      </div>
    );
  }

  if (loading) {
    return (
      <div className="h-[calc(100vh-3.5rem)] flex items-center justify-center">
        <div className="text-center">
          <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-[#7645d9]/12 to-[#1fc7d4]/12 border border-[#7645d9]/20 flex items-center justify-center mx-auto mb-4 shadow-sm">
            <Loader2 className="w-6 h-6 text-[#7645d9] animate-spin" />
          </div>
          <p className="text-sm font-semibold text-foreground/60">Loading conversations...</p>
          <p className="text-xs text-muted-foreground/40 mt-1">Just a moment</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-[calc(100vh-3.5rem)] overflow-hidden">
      <ChatInbox
        topics={topics}
        initConvos={convos}
        userAddr={getWalletAddress() ?? undefined}
      />
    </div>
  );
}
