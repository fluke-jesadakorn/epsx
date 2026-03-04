'use client';

import {
  createConversationAction,
  getMessagesAction,
  listConversationsAction,
  markConversationReadAction,
  notifyTypingAction,
  sendMessageAction,
  updateConversationStatusAction,
  uploadAttachmentAction,
} from '@/app/actions/chat';
import type { ChatConversation, ChatMessage, ChatTopic } from '@/shared/api/chat';
import type { ChatSSEEvent } from '@/shared/hooks/use-chat-sse';
import { useChatSSE } from '@/shared/hooks/use-chat-sse';
import {
  ArrowLeft,
  CheckCircle,
  Headset,
  Inbox,
  Loader2,
  MessageCircle,
  Plus,
  Tag,
} from 'lucide-react';
import { TurnstileWidget } from '@/shared/components/turnstile-widget';
import { useCallback, useEffect, useRef, useState, useTransition } from 'react';
import { ChatInput } from './chat-input';
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
  if (days > 0) { return `${days}d ago`; }
  if (hrs > 0) { return `${hrs}h ago`; }
  if (mins > 0) { return `${mins}m ago`; }
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
  const [mobileView, setMobileView] = useState<'list' | 'chat'>('list');
  const [isPending, startTransition] = useTransition();
  const markedRef = useRef<string | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [widgetKey, setWidgetKey] = useState(0);

  const handleTokenUsed = useCallback(() => {
    setToken(null);
    setWidgetKey((k) => k + 1);
  }, []);

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

  const handleSend = useCallback((content: string, turnstileToken: string) => {
    if (!content || !selected || isPending) { return; }
    const doSend = async () => {
      const msg = await sendMessageAction(selected, content, turnstileToken);
      if (msg) { setMsgs(prev => [...prev, msg]); }
    };
    startTransition(() => { void doSend(); });
  }, [selected, isPending]);

  const handleResolve = useCallback(() => {
    if (!selected || isPending) { return; }
    const doResolve = async () => {
      const updated = await updateConversationStatusAction(selected, 'resolved');
      if (updated) { setConvos(prev => prev.map(c => (c.id === selected ? updated : c))); }
    };
    startTransition(() => { void doResolve(); });
  }, [selected, isPending]);

  const handleTyping = useCallback((isTyping: boolean) => {
    if (!selected) { return; }
    void notifyTypingAction(selected, isTyping);
  }, [selected]);

  const handleUpload = useCallback((file: File) => {
    if (!selected) { return; }
    void (async () => {
      const formData = new FormData();
      formData.append('file', file);
      const msg = await uploadAttachmentAction(selected, formData);
      if (msg) { setMsgs(prev => [...prev, msg]); }
    })();
  }, [selected]);

  const handleCreate = useCallback(
    async ({ topicId, subject, message, turnstileToken, file }: { topicId: string; subject: string; message: string; turnstileToken?: string; file?: File }) => {
      const convo = await createConversationAction({ topicId, subject, message, turnstileToken });
      if (convo) {
        if (file) {
          const formData = new FormData();
          formData.append('file', file);
          void uploadAttachmentAction(convo.id, formData);
        }
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

  const filtered = convos.filter(c => {
    if (statusFilter !== '' && c.status !== statusFilter) { return false; }
    if (topicFilter !== '' && c.topic_id !== topicFilter) { return false; }
    return true;
  });

  const selectedConv = convos.find(c => c.id === selected);
  const selectedTopic = selectedConv ? topics.find(t => t.id === selectedConv.topic_id) : null;
  const canResolve =
    selectedConv !== undefined &&
    selectedConv.status !== 'resolved' &&
    selectedConv.status !== 'closed';

  return (
    <div className="h-[calc(100vh-5rem)] md:h-[calc(100vh-6rem)] flex flex-col md:flex-row md:gap-3">
      <div aria-hidden className="absolute opacity-0 pointer-events-none">
        <TurnstileWidget
          key={widgetKey}
          action="chat"
          onSuccess={setToken}
          onExpire={() => setToken(null)}
        />
      </div>
      {/* Left: Conversation List */}
      <div className={`w-full md:w-[320px] md:flex-shrink-0 flex flex-col bg-white dark:bg-slate-900/80 border border-slate-200 dark:border-slate-700 rounded-2xl overflow-hidden ${mobileView === 'chat' ? 'hidden md:flex' : 'flex'}`}>
        {/* Panel Header */}
        <div className="flex items-center justify-between px-3 py-2.5 border-b border-slate-200 dark:border-slate-700">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center shadow-sm shadow-blue-500/20 shrink-0">
              <Headset className="w-3.5 h-3.5 text-white" />
            </div>
            <div>
              <h2 className="font-bold text-sm tracking-tight leading-tight">Support</h2>
              <div className="flex items-center gap-1">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                <span className="text-[10px] text-muted-foreground/60">Online</span>
              </div>
            </div>
          </div>
          <span className="text-[10px] font-medium text-muted-foreground/40 bg-slate-100 dark:bg-slate-800/60 px-1.5 py-0.5 rounded-md">
            {filtered.length}
          </span>
        </div>

        {/* Filter Bar */}
        <div className="flex items-center gap-1.5 px-2.5 py-2 border-b border-slate-100 dark:border-slate-800">
          <select
            value={statusFilter}
            onChange={e => setStatusFilter(e.target.value)}
            className="flex-1 px-2 py-1 text-[11px] font-medium bg-slate-100 dark:bg-slate-800/50 border border-slate-200 dark:border-white/8 rounded-lg text-foreground focus:outline-none focus:border-blue-500/50 transition-all cursor-pointer"
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
            className="flex-1 px-2 py-1 text-[11px] font-medium bg-slate-100 dark:bg-slate-800/50 border border-slate-200 dark:border-white/8 rounded-lg text-foreground focus:outline-none focus:border-blue-500/50 transition-all cursor-pointer"
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
        <div className="flex-1 overflow-y-auto scrollbar-thin">
          {filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <div className="w-10 h-10 rounded-xl bg-slate-100 dark:bg-slate-800/40 flex items-center justify-center mb-2">
                <Inbox className="w-5 h-5 text-muted-foreground/40" />
              </div>
              <p className="text-xs font-medium text-muted-foreground mb-0.5">No conversations</p>
              <p className="text-[10px] text-muted-foreground/50">Start a new one below</p>
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
                    setMobileView('chat');
                  }}
                  className={`w-full text-left px-3 py-2.5 border-b border-slate-100 dark:border-slate-800 transition-all ${isSel
                    ? 'bg-blue-50 dark:bg-blue-500/10 border-l-2 border-l-blue-500'
                    : 'hover:bg-slate-50 dark:hover:bg-slate-800/60'
                    }`}
                >
                  <div className="flex items-start justify-between gap-2 mb-1">
                    <p className={`text-[13px] leading-snug line-clamp-1 ${unread ? 'font-bold' : 'font-medium text-foreground/80'}`}>
                      {c.subject}
                    </p>
                    {unread && (
                      <div className="flex-shrink-0 min-w-[18px] h-[18px] px-1 rounded-full bg-blue-500 text-white text-[9px] font-bold flex items-center justify-center">
                        {c.unread_user > 9 ? '9+' : c.unread_user}
                      </div>
                    )}
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <ChatStatusBadge status={c.status} />
                      {topic && (
                        <span className="text-[10px] font-medium text-blue-400">{topic.label}</span>
                      )}
                    </div>
                    <span className="text-[10px] text-muted-foreground/50">{timeAgo(c.last_message_at)}</span>
                  </div>
                </button>
              );
            })
          )}
        </div>

        {/* New Conversation */}
        <div className="p-2 border-t border-slate-200 dark:border-slate-700">
          <button
            onClick={() => {
              setShowNew(true);
              setSelected(null);
              setMobileView('chat');
            }}
            className="w-full py-2 rounded-xl bg-gradient-to-r from-blue-500 to-blue-600 text-white font-semibold text-xs hover:from-blue-400 hover:to-blue-500 transition-all shadow-sm shadow-blue-500/20 flex items-center justify-center gap-1.5"
          >
            <Plus className="w-3.5 h-3.5" />
            New Conversation
          </button>
        </div>
      </div>

      {/* Right: Conversation View */}
      <div className={`flex-1 min-w-0 border border-slate-200 dark:border-slate-700 rounded-2xl bg-white dark:bg-slate-900/80 overflow-hidden flex flex-col ${mobileView === 'list' ? 'hidden md:flex' : 'flex'}`}>
        {showNew ? (
          <div className="flex-1 flex flex-col overflow-hidden">
            <button
              onClick={handleMobileBack}
              className="md:hidden flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground px-4 pt-3 w-fit transition-colors"
            >
              <ArrowLeft className="w-3.5 h-3.5" />
              Back
            </button>
            <ChatTopicSelector topics={topics} onSelect={handleCreate} turnstileToken={token} />
          </div>
        ) : selectedConv ? (
          <>
            {/* Header */}
            <div className="border-b border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-900">
              <div className="h-0.5 bg-gradient-to-r from-blue-500 via-indigo-500 to-blue-400" />
              <div className="p-4">
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
            <div>
              {canResolve && (
                <div className="px-3 pt-2 bg-slate-50 dark:bg-slate-950/40 border-t border-slate-200 dark:border-slate-700">
                  <button
                    onClick={handleResolve}
                    disabled={isPending}
                    className="px-3 py-1.5 text-[11px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-lg hover:bg-emerald-500/20 transition-all disabled:opacity-40 flex items-center gap-1.5 uppercase tracking-wide"
                  >
                    <CheckCircle className="w-3.5 h-3.5" /> Resolve
                  </button>
                </div>
              )}
              <ChatInput
                onSend={handleSend}
                onUpload={handleUpload}
                onTyping={handleTyping}
                disabled={isPending || selectedConv.status === 'closed'}
                placeholder={selectedConv.status === 'closed' ? 'Conversation is closed' : 'Type your reply...'}
                turnstileToken={token}
                onTokenUsed={handleTokenUsed}
              />
            </div>
          </>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center px-8">
            <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-blue-500/10 to-indigo-500/10 border border-blue-500/15 flex items-center justify-center mb-3">
              <MessageCircle className="w-7 h-7 text-blue-400/30" />
            </div>
            <p className="text-sm font-medium text-foreground/60 mb-1">Select a conversation</p>
            <p className="text-[11px] text-muted-foreground/40">Choose from the left panel</p>
          </div>
        )}
      </div>
    </div>
  );
}
