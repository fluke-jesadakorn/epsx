/* eslint-disable max-lines-per-function */
'use client';

import {
  createConversationAction,
  getMessagesAction,
  listConversationsAction,
  markConversationReadAction,
  sendMessageAction,
  updateConversationStatusAction,
} from '@/app/actions/chat';
import type { ChatConversation, ChatMessage, ChatTopic } from '@/shared/api/chat';
import type { ChatSSEEvent } from '@/shared/hooks/use-chat-sse';
import { useChatSSE } from '@/shared/hooks/use-chat-sse';
import {
  ArrowLeft,
  CheckCircle,
  Clock,
  Inbox,
  Loader2,
  MessageCircle,
  Plus,
  Send,
  SlidersHorizontal,
  Tag,
} from 'lucide-react';
import { useCallback, useEffect, useRef, useState, useTransition, type KeyboardEvent } from 'react';
import { ChatMessageList } from './chat-message-list';
import { ChatStatusBadge } from './chat-status-badge';
import { ChatTopicSelector } from './chat-topic-selector';

interface Props {
  topics: ChatTopic[];
  initConvos: ChatConversation[];
  userAddr?: string;
}

function timeAgo(date: string): string {
  const diff = Date.now() - new Date(date).getTime();
  const mins = Math.floor(diff / 60000);
  const hrs = Math.floor(mins / 60);
  const days = Math.floor(hrs / 24);
  if (days > 0) return `${days}d ago`;
  if (hrs > 0) return `${hrs}h ago`;
  if (mins > 0) return `${mins}m ago`;
  return 'Just now';
}

export function ChatInbox({ topics, initConvos, userAddr }: Props) {
  const [convos, setConvos] = useState(initConvos);
  const [selected, setSelected] = useState<string | null>(initConvos[0]?.id ?? null);
  const [statusFilter, setStatusFilter] = useState('');
  const [topicFilter, setTopicFilter] = useState('');
  const [showNew, setShowNew] = useState(initConvos.length === 0);
  const [msgs, setMsgs] = useState<ChatMessage[]>([]);
  const [loadingMsgs, setLoadingMsgs] = useState(false);
  const [replyMsg, setReplyMsg] = useState('');
  const [mobileView, setMobileView] = useState<'list' | 'chat'>('list');
  const [isPending, startTransition] = useTransition();
  const markedRef = useRef<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const reloadConvos = useCallback(async () => {
    const data = await listConversationsAction();
    setConvos(Array.isArray(data) ? data : []);
  }, []);

  // Load messages on selection change
  useEffect(() => {
    if (!selected || showNew) {
      setMsgs([]);
      return;
    }
    const load = async () => {
      setLoadingMsgs(true);
      const data = await getMessagesAction(selected);
      setMsgs(Array.isArray(data) ? data : []);
      setLoadingMsgs(false);
      if (markedRef.current !== selected) {
        markedRef.current = selected;
        void markConversationReadAction(selected);
      }
    };
    void load();
  }, [selected, showNew]);

  // Auto-resize textarea
  useEffect(() => {
    const el = textareaRef.current;
    if (el) {
      el.style.height = 'auto';
      el.style.height = `${Math.min(el.scrollHeight, 120)}px`;
    }
  }, [replyMsg]);

  // SSE real-time
  const handleSSE = useCallback((evt: ChatSSEEvent) => {
    if (evt.type === 'new_message' && evt.message) {
      if (evt.conversation_id === selected) {
        setMsgs(prev =>
          prev.some(m => m.id === evt.message?.id) ? prev : [...prev, evt.message as ChatMessage]
        );
        void markConversationReadAction(selected);
      } else {
        setConvos(prev =>
          prev.map(c =>
            c.id === evt.conversation_id ? { ...c, unread_user: c.unread_user + 1 } : c
          )
        );
      }
    }
    if (evt.type === 'status_changed') {
      void reloadConvos();
    }
  }, [selected, reloadConvos]);

  useChatSSE({ enabled: true, mode: 'user', onEvent: handleSSE });

  const handleSend = useCallback(() => {
    const content = replyMsg.trim();
    if (!content || !selected || isPending) return;
    startTransition(() => {
      void (async () => {
        const msg = await sendMessageAction(selected, content);
        if (msg) setMsgs(prev => [...prev, msg]);
        setReplyMsg('');
      })();
    });
  }, [replyMsg, selected, isPending]);

  const handleResolve = useCallback(() => {
    if (!selected || isPending) return;
    startTransition(() => {
      void (async () => {
        const updated = await updateConversationStatusAction(selected, 'resolved');
        if (updated) setConvos(prev => prev.map(c => (c.id === selected ? updated : c)));
      })();
    });
  }, [selected, isPending]);

  const handleCreate = useCallback(
    async (topicId: string, subject: string, message: string, turnstileToken?: string) => {
      const convo = await createConversationAction(topicId, subject, message, turnstileToken);
      if (convo) {
        await reloadConvos();
        setSelected(convo.id);
        setShowNew(false);
        setMobileView('chat');
      }
    },
    [reloadConvos]
  );

  const handleMobileBack = useCallback(() => {
    setMobileView('list');
    setSelected(null);
    setShowNew(false);
  }, []);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const filtered = convos.filter(c => {
    if (statusFilter !== '' && c.status !== statusFilter) return false;
    if (topicFilter !== '' && c.topic_id !== topicFilter) return false;
    return true;
  });

  const selectedConv = convos.find(c => c.id === selected);
  const selectedTopic = selectedConv ? topics.find(t => t.id === selectedConv.topic_id) : null;
  const canResolve =
    selectedConv !== undefined &&
    selectedConv.status !== 'resolved' &&
    selectedConv.status !== 'closed';
  const canSend = replyMsg.trim() !== '' && !isPending && selectedConv?.status !== 'closed';

  return (
    <div className="h-[calc(100vh-8rem)] md:h-[calc(100vh-12rem)] flex flex-col md:flex-row md:gap-4">
      {/* Left: Conversation List */}
      <div className={`w-full md:w-[360px] md:flex-shrink-0 flex flex-col ${mobileView === 'chat' ? 'hidden md:flex' : 'flex'}`}>
        {/* Filter Bar */}
        <div className="flex items-center gap-2 mb-3 p-2.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl">
          <SlidersHorizontal className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          <select
            value={statusFilter}
            onChange={e => setStatusFilter(e.target.value)}
            className="flex-1 px-2.5 py-1.5 text-xs font-medium bg-slate-100 dark:bg-slate-800/50 border border-slate-200 dark:border-white/8 rounded-lg text-foreground focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 transition-all cursor-pointer"
          >
            <option value="">All Status</option>
            <option value="open">Open</option>
            <option value="in_progress">In Progress</option>
            <option value="resolved">Resolved</option>
            <option value="closed">Closed</option>
          </select>
          <select
            value={topicFilter}
            onChange={e => setTopicFilter(e.target.value)}
            className="flex-1 px-2.5 py-1.5 text-xs font-medium bg-slate-100 dark:bg-slate-800/50 border border-slate-200 dark:border-white/8 rounded-lg text-foreground focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 transition-all cursor-pointer"
          >
            <option value="">All Topics</option>
            {topics.map(t => (
              <option key={t.id} value={t.id}>
                {t.label}
              </option>
            ))}
          </select>
        </div>

        {/* Cards */}
        <div className="flex-1 overflow-y-auto space-y-2 pr-1 scrollbar-thin">
          {filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <div className="w-12 h-12 rounded-xl bg-slate-100 dark:bg-slate-800/40 flex items-center justify-center mb-3 border border-slate-200 dark:border-slate-700">
                <Inbox className="w-6 h-6 text-muted-foreground/40" />
              </div>
              <p className="text-sm font-medium text-muted-foreground mb-1">No conversations</p>
              <p className="text-xs text-muted-foreground/50">Start a new conversation below</p>
            </div>
          ) : (
            filtered.map(c => {
              const topic = topics.find(t => t.id === c.topic_id);
              const isSel = selected === c.id && !showNew;
              const unread = c.unread_user > 0;
              return (
                <button
                  key={c.id}
                  onClick={() => {
                    setSelected(c.id);
                    setShowNew(false);
                    setReplyMsg('');
                    setMobileView('chat');
                  }}
                  className={`w-full text-left p-3.5 rounded-xl transition-all ${isSel
                      ? 'bg-blue-50 dark:bg-blue-500/10 border border-blue-200 dark:border-blue-500/25 shadow-sm shadow-blue-500/5'
                      : 'bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800/80 hover:border-slate-300 dark:hover:border-slate-700'
                    }`}
                >
                  <div className="flex items-start justify-between gap-2 mb-2">
                    <p
                      className={`text-sm leading-snug line-clamp-1 ${unread ? 'font-bold' : 'font-semibold text-foreground/80'}`}
                    >
                      {c.subject}
                    </p>
                    {unread && (
                      <div className="flex-shrink-0 min-w-[22px] h-[22px] px-1 rounded-full bg-gradient-to-r from-blue-500 to-blue-600 text-white text-[10px] font-bold flex items-center justify-center shadow-sm shadow-blue-500/30">
                        {c.unread_user > 9 ? '9+' : c.unread_user}
                      </div>
                    )}
                  </div>
                  {topic && (
                    <div className="flex items-center gap-1.5 mb-2">
                      <Tag className="w-3 h-3 text-blue-400" />
                      <span className="text-[11px] font-semibold text-blue-400">{topic.label}</span>
                    </div>
                  )}
                  <div className="flex items-center justify-between">
                    <ChatStatusBadge status={c.status} />
                    <div className="flex items-center gap-1 text-[10px] text-muted-foreground/60">
                      <Clock className="w-2.5 h-2.5" />
                      {timeAgo(c.last_message_at)}
                    </div>
                  </div>
                </button>
              );
            })
          )}
        </div>

        {/* New Conversation */}
        <button
          onClick={() => {
            setShowNew(true);
            setSelected(null);
            setMobileView('chat');
          }}
          className="mt-3 w-full py-2.5 rounded-xl bg-gradient-to-r from-blue-500 to-blue-600 text-white font-semibold text-sm hover:from-blue-400 hover:to-blue-500 transition-all shadow-sm shadow-blue-500/20 flex items-center justify-center gap-2"
        >
          <Plus className="w-4 h-4" />
          New Conversation
        </button>
      </div>

      {/* Right: Conversation View */}
      <div className={`flex-1 border border-slate-200 dark:border-slate-700 rounded-2xl bg-white dark:bg-slate-900/80 overflow-hidden flex flex-col ${mobileView === 'list' ? 'hidden md:flex' : 'flex'}`}>
        {showNew ? (
          <div className="flex-1 flex flex-col overflow-hidden">
            <button
              onClick={handleMobileBack}
              className="md:hidden flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground px-4 pt-3 w-fit transition-colors"
            >
              <ArrowLeft className="w-3.5 h-3.5" />
              Back
            </button>
            <ChatTopicSelector topics={topics} onSelect={handleCreate} />
          </div>
        ) : selectedConv ? (
          <>
            {/* Header */}
            <div className="border-b border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-900 p-4">
              <div className="flex items-start justify-between gap-4">
                <div className="flex items-center gap-2 flex-1 min-w-0">
                  <button
                    onClick={handleMobileBack}
                    className="md:hidden w-8 h-8 rounded-lg hover:bg-muted flex items-center justify-center transition-colors shrink-0"
                  >
                    <ArrowLeft className="w-4 h-4 text-muted-foreground" />
                  </button>
                  <div className="min-w-0">
                    <h2 className="text-lg font-bold mb-1 truncate">{selectedConv.subject}</h2>
                    {selectedTopic && (
                      <div className="flex items-center gap-1.5">
                        <Tag className="w-3.5 h-3.5 text-blue-400" />
                        <span className="text-xs font-semibold text-blue-400">
                          {selectedTopic.label}
                        </span>
                      </div>
                    )}
                  </div>
                </div>
                <ChatStatusBadge status={selectedConv.status} />
              </div>
            </div>

            {/* Messages */}
            {loadingMsgs ? (
              <div className="flex-1 flex items-center justify-center">
                <Loader2 className="w-6 h-6 text-blue-400 animate-spin" />
              </div>
            ) : (
              <ChatMessageList msgs={msgs} userAddr={userAddr} />
            )}

            {/* Reply */}
            <div className="border-t border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-950/40 p-3">
              {canResolve && (
                <div className="flex gap-2 mb-2">
                  <button
                    onClick={handleResolve}
                    disabled={isPending}
                    className="px-3 py-1.5 text-[11px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-lg hover:bg-emerald-500/20 transition-all disabled:opacity-40 flex items-center gap-1.5 uppercase tracking-wide"
                  >
                    <CheckCircle className="w-3.5 h-3.5" />
                    Resolve
                  </button>
                </div>
              )}
              <div className="flex items-end gap-2">
                <div className="flex-1 bg-slate-100 dark:bg-slate-800/30 border border-slate-200 dark:border-white/8 rounded-xl focus-within:border-blue-500/30 focus-within:ring-1 focus-within:ring-blue-500/10 transition-all overflow-hidden">
                  <textarea
                    ref={textareaRef}
                    value={replyMsg}
                    onChange={e => setReplyMsg(e.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Type your reply..."
                    disabled={isPending || selectedConv.status === 'closed'}
                    className="w-full px-4 py-3 bg-transparent text-sm placeholder:text-muted-foreground/40 focus:outline-none resize-none disabled:opacity-40 disabled:cursor-not-allowed"
                    rows={2}
                  />
                </div>
                <button
                  onClick={handleSend}
                  disabled={!canSend}
                  className={`px-4 py-3 rounded-xl flex items-center justify-center transition-all ${canSend
                      ? 'bg-gradient-to-r from-blue-500 to-blue-600 text-white hover:from-blue-400 hover:to-blue-500 shadow-sm shadow-blue-500/20'
                      : 'bg-muted text-muted-foreground/30 cursor-not-allowed'
                    }`}
                >
                  <Send className="w-4 h-4" />
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <div className="w-16 h-16 rounded-2xl bg-slate-100 dark:bg-slate-800/40 flex items-center justify-center mb-4 border border-slate-200 dark:border-slate-700">
              <MessageCircle className="w-8 h-8 text-muted-foreground/20" />
            </div>
            <p className="text-sm font-medium text-muted-foreground mb-1">Select a conversation</p>
            <p className="text-xs text-muted-foreground/40">
              Choose from the left panel to view details
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
