/* eslint-disable max-lines-per-function */
'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
import { ArrowLeft, User, Bot, Headset, Wallet, Tag, UserCheck, MessageCircle } from 'lucide-react';
import type { ChatConversation, ChatMessage, ChatTopic } from '@/shared/api/chat';
import { ChatStatusBadge } from './chat-status-badge';
import { ChatReplyInput } from './chat-reply-input';
import { getMessages, sendReply, assignAgent, updateStatus, markAsRead } from '@/app/actions/chat';
import { useSharedAuth } from '@/shared/components/auth';
import { useChatSSE } from '@/shared/hooks/use-chat-sse';
import type { ChatSSEEvent } from '@/shared/hooks/use-chat-sse';

interface Props {
  conv: ChatConversation;
  topics: ChatTopic[];
  onUpdate: () => void;
  onBack?: () => void;
}

function truncAddr(addr: string): string {
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
}

function formatTime(date: string): string {
  return new Date(date).toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatDate(date: string): string {
  return new Date(date).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

export function ChatConversationView({ conv, topics, onUpdate, onBack }: Props) {
  const [msgs, setMsgs] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);
  const markedRef = useRef<string | null>(null);
  const { user } = useSharedAuth();
  const walletAddress = user?.wallet_address;

  const topic = topics.find(t => t.id === conv.topic_id);

  const loadMsgs = useCallback(async () => {
    setLoading(true);
    const res = await getMessages(conv.id);
    if (res.success && Array.isArray(res.data)) {
      setMsgs(res.data);
    }
    setLoading(false);
  }, [conv.id]);

  useEffect(() => {
    void loadMsgs();
    // Mark as read once per conversation
    if (markedRef.current !== conv.id) {
      markedRef.current = conv.id;
      void markAsRead(conv.id);
    }
  }, [conv.id, loadMsgs]);

  useEffect(() => {
    const ref = scrollRef.current;
    if (ref !== null) {
      ref.scrollTop = ref.scrollHeight;
    }
  }, [msgs]);

  // SSE: real-time message delivery & status updates
  const handleSSE = useCallback((evt: ChatSSEEvent) => {
    if (evt.type === 'new_message' && evt.message && evt.conversation_id === conv.id) {
      setMsgs((prev) => prev.some((m) => m.id === evt.message?.id) ? prev : [...prev, evt.message as ChatMessage]);
      void markAsRead(conv.id);
    }
    if ((evt.type === 'status_changed' || evt.type === 'agent_assigned') && evt.conversation_id === conv.id) {
      onUpdate();
    }
  }, [conv.id, onUpdate]);

  useChatSSE({ enabled: true, mode: 'admin', onEvent: handleSSE });

  const handleSend = async (content: string) => {
    const res = await sendReply(conv.id, content);
    if (res.success && res.data) {
      setMsgs(prev => [...prev, res.data as ChatMessage]);
      onUpdate();
    }
  };

  const handleResolve = async () => {
    await updateStatus(conv.id, 'resolved');
    onUpdate();
  };

  const handleClose = async () => {
    await updateStatus(conv.id, 'closed');
    onUpdate();
  };

  const handleAssign = async (addr: string) => {
    await assignAgent(conv.id, addr);
    onUpdate();
  };

  const canResolve = conv.status !== 'resolved';
  const canClose = conv.status !== 'closed';

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-4 backdrop-blur-sm">
        <div className="flex items-start justify-between gap-4 mb-3">
          <div className="flex items-center gap-2 flex-1 min-w-0">
            {onBack && (
              <button
                onClick={onBack}
                className="md:hidden w-8 h-8 rounded-lg hover:bg-muted flex items-center justify-center transition-colors shrink-0"
              >
                <ArrowLeft className="w-4 h-4 text-muted-foreground" />
              </button>
            )}
            <div className="min-w-0">
              <h2 className="text-lg font-bold text-foreground mb-1 truncate">{conv.subject}</h2>
              <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
                <div className="flex items-center gap-1.5 text-muted-foreground">
                  <Wallet className="w-3.5 h-3.5" />
                  <span className="text-xs font-mono">{truncAddr(conv.wallet_address)}</span>
                </div>
                {topic && (
                  <div className="flex items-center gap-1.5">
                    <Tag className="w-3.5 h-3.5 text-violet-400" />
                    <span className="text-xs font-semibold text-violet-400">{topic.label}</span>
                  </div>
                )}
                {conv.assigned_agent !== null && conv.assigned_agent !== '' && (
                  <div className="flex items-center gap-1.5 text-muted-foreground">
                    <UserCheck className="w-3.5 h-3.5 text-cyan-400" />
                    <span className="text-xs font-mono">{truncAddr(conv.assigned_agent)}</span>
                  </div>
                )}
              </div>
            </div>
          </div>
          <ChatStatusBadge status={conv.status} />
        </div>
      </div>

      {/* Messages */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-5 space-y-4 bg-gray-50 dark:bg-slate-950/20"
      >
        {loading ? (
          <div className="flex flex-col items-center justify-center h-full">
            <div className="w-10 h-10 rounded-xl bg-violet-500/10 flex items-center justify-center mb-3">
              <MessageCircle className="w-5 h-5 text-violet-400 animate-pulse" />
            </div>
            <p className="text-sm text-muted-foreground">Loading messages...</p>
          </div>
        ) : msgs.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <div className="w-12 h-12 rounded-xl bg-gray-100 dark:bg-slate-800/40 flex items-center justify-center mb-3 border border-gray-200 dark:border-slate-700">
              <MessageCircle className="w-6 h-6 text-muted-foreground/30" />
            </div>
            <p className="text-sm text-muted-foreground">No messages yet</p>
          </div>
        ) : (
          msgs.map((m, i) => {
            const isAgent = m.sender_type === 'agent';
            const isSystem = m.sender_type === 'system';
            const isAi = m.sender_type === 'ai';
            const isRight = isAgent || isAi;

            // Date separator
            const showDate = i === 0 || formatDate(msgs[i - 1]?.created_at ?? '') !== formatDate(m.created_at);

            if (isSystem) {
              return (
                <div key={m.id}>
                  {showDate && (
                    <div className="flex items-center gap-3 my-4">
                      <div className="flex-1 h-px bg-gray-200 dark:bg-white/[0.04]" />
                      <span className="text-[10px] text-muted-foreground/40 font-medium uppercase tracking-widest">{formatDate(m.created_at)}</span>
                      <div className="flex-1 h-px bg-gray-200 dark:bg-white/[0.04]" />
                    </div>
                  )}
                  <div className="flex justify-center">
                    <div className="px-3 py-1.5 bg-gray-100 dark:bg-slate-800/30 border border-gray-200 dark:border-slate-700 rounded-full text-[11px] text-muted-foreground/60">
                      {m.content}
                    </div>
                  </div>
                </div>
              );
            }

            return (
              <div key={m.id}>
                {showDate && (
                  <div className="flex items-center gap-3 my-4">
                    <div className="flex-1 h-px bg-white dark:bg-white/[0.04]" />
                    <span className="text-[10px] text-muted-foreground/40 font-medium uppercase tracking-widest">{formatDate(m.created_at)}</span>
                    <div className="flex-1 h-px bg-white dark:bg-white/[0.04]" />
                  </div>
                )}
                <div className={`flex gap-2.5 ${isRight ? 'justify-end' : 'justify-start'}`}>
                  {/* Avatar for left-side messages */}
                  {!isRight && (
                    <div className="shrink-0 w-8 h-8 rounded-full bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700 flex items-center justify-center mt-5">
                      <User className="w-4 h-4 text-muted-foreground/60" />
                    </div>
                  )}

                  <div className={`max-w-[70%] flex flex-col ${isRight ? 'items-end' : 'items-start'}`}>
                    {/* Sender label */}
                    <div className="flex items-center gap-1.5 mb-1 px-1">
                      {isAi ? (
                        <Bot className="w-3 h-3 text-purple-400" />
                      ) : isAgent ? (
                        <Headset className="w-3 h-3 text-cyan-400" />
                      ) : null}
                      <span className="text-[10px] font-bold text-muted-foreground/60 uppercase tracking-wider">
                        {isAi ? 'AI' : isAgent ? 'Agent' : 'User'}
                      </span>
                      <span className="text-[10px] text-muted-foreground/30">
                        {formatTime(m.created_at)}
                      </span>
                    </div>

                    {/* Message bubble */}
                    <div
                      className={`rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${
                        isRight
                          ? isAi
                            ? 'bg-purple-500/10 border border-purple-500/20 text-foreground rounded-br-md'
                            : 'bg-gradient-to-br from-violet-500/15 to-purple-500/10 border border-violet-500/20 text-foreground rounded-br-md'
                          : 'bg-gray-100 dark:bg-slate-800/50 border border-gray-200 dark:border-slate-700 text-foreground rounded-bl-md'
                      }`}
                    >
                      <p className="whitespace-pre-wrap break-words">{m.content}</p>
                    </div>
                  </div>

                  {/* Avatar for right-side messages */}
                  {isRight && (
                    <div className={`shrink-0 w-8 h-8 rounded-full flex items-center justify-center mt-5 ${
                      isAi ? 'bg-purple-500/15 border border-purple-500/20' : 'bg-cyan-500/15 border border-cyan-500/20'
                    }`}>
                      {isAi ? (
                        <Bot className="w-4 h-4 text-purple-400" />
                      ) : (
                        <Headset className="w-4 h-4 text-cyan-400" />
                      )}
                    </div>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Reply Input */}
      <ChatReplyInput
        onSend={handleSend}
        onResolve={canResolve ? handleResolve : undefined}
        onClose={canClose ? handleClose : undefined}
        onAssign={handleAssign}
        myWallet={(walletAddress as string | undefined) ?? ''}
        assignedAgent={conv.assigned_agent}
      />
    </div>
  );
}
