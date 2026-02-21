'use client';

import { useState, useEffect, useCallback } from 'react';
import type { ChatConversation, ChatTopic } from '@/shared/api/chat';
import { ChatConversationCard } from './chat-conversation-card';
import { ChatConversationView } from './chat-conversation-view';
import { ChatFilterBar } from './chat-filter-bar';
import { listAllConversations } from '@/app/actions/chat';
import { useChatSSE } from '@/shared/hooks/use-chat-sse';
import type { ChatSSEEvent } from '@/shared/hooks/use-chat-sse';
import { MessageCircle, Inbox } from 'lucide-react';

interface Props {
  initConvs: ChatConversation[];
  topics: ChatTopic[];
}

export function ChatInbox({ initConvs, topics }: Props) {
  const [convs, setConvs] = useState(initConvs);
  const [selected, setSelected] = useState<string | null>(initConvs[0]?.id ?? null);
  const [status, setStatus] = useState('');
  const [topicId, setTopicId] = useState('');
  const [loading, setLoading] = useState(false);

  const loadConvs = useCallback(async () => {
    setLoading(true);
    const res = await listAllConversations(
      status !== '' ? status : undefined,
      topicId !== '' ? topicId : undefined,
      undefined
    );
    if (res.success && Array.isArray(res.data)) {
      setConvs(res.data);
      // Auto-select first if none selected
      const first = res.data[0];
      if (selected === null && res.data.length > 0 && first) {
        setSelected(first.id);
      }
    }
    setLoading(false);
  }, [status, topicId, selected]);

  useEffect(() => {
    void loadConvs();
  }, [loadConvs]);

  // SSE: real-time updates (replaces 10s polling)
  const handleSSE = useCallback((evt: ChatSSEEvent) => {
    if (evt.type === 'new_conversation' || evt.type === 'status_changed' || evt.type === 'agent_assigned') {
      void loadConvs();
    }
    if (evt.type === 'new_message') {
      // Increment unread for the conversation
      setConvs((prev) => prev.map((c) =>
        c.id === evt.conversation_id ? { ...c, unread_agent: c.unread_agent + 1 } : c
      ));
    }
  }, [loadConvs]);

  useChatSSE({ enabled: true, mode: 'admin', onEvent: handleSSE });

  const handleUpdate = useCallback(() => {
    void loadConvs();
  }, [loadConvs]);

  const selectedConv = convs.find(c => c.id === selected);

  return (
    <div className="h-[calc(100vh-14rem)] flex gap-4">
      {/* Left: Conversation List */}
      <div className="w-[360px] flex-shrink-0 flex flex-col">
        <ChatFilterBar
          status={status}
          topicId={topicId}
          topics={topics}
          onStatusChange={setStatus}
          onTopicChange={setTopicId}
        />
        <div className="flex-1 overflow-y-auto space-y-2 pr-1 scrollbar-thin">
          {loading && convs.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <div className="w-10 h-10 rounded-xl bg-violet-500/10 flex items-center justify-center mb-3">
                <MessageCircle className="w-5 h-5 text-violet-400 animate-pulse" />
              </div>
              <p className="text-sm text-muted-foreground">Loading conversations...</p>
            </div>
          ) : convs.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <div className="w-12 h-12 rounded-xl bg-gray-100 dark:bg-slate-800/60 flex items-center justify-center mb-3 border border-gray-200 dark:border-slate-700">
                <Inbox className="w-6 h-6 text-muted-foreground/40" />
              </div>
              <p className="text-sm font-medium text-muted-foreground mb-1">No conversations</p>
              <p className="text-xs text-muted-foreground/50">Conversations will appear here</p>
            </div>
          ) : (
            convs.map(c => (
              <ChatConversationCard
                key={c.id}
                conv={c}
                topics={topics}
                selected={selected === c.id}
                onClick={() => {
                  setSelected(c.id);
                }}
              />
            ))
          )}
        </div>
      </div>

      {/* Right: Conversation View */}
      <div className="flex-1 border border-gray-200 dark:border-slate-700 rounded-2xl bg-white dark:bg-slate-900/80 overflow-hidden backdrop-blur-sm">
        {selectedConv ? (
          <ChatConversationView
            conv={selectedConv}
            topics={topics}
            onUpdate={handleUpdate}
          />
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <div className="w-16 h-16 rounded-2xl bg-gray-100 dark:bg-slate-800/40 flex items-center justify-center mb-4 border border-gray-200 dark:border-slate-700">
              <MessageCircle className="w-8 h-8 text-muted-foreground/20" />
            </div>
            <p className="text-sm font-medium text-muted-foreground mb-1">Select a conversation</p>
            <p className="text-xs text-muted-foreground/40">Choose from the left panel to view details</p>
          </div>
        )}
      </div>
    </div>
  );
}
