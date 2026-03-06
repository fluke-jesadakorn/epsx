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
  Search,
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
  if (days > 0) { return `${days}d`; }
  if (hrs > 0) { return `${hrs}h`; }
  if (mins > 0) { return `${mins}m`; }
  return 'now';
}

export function ChatInbox({ topics, initConvos, userAddr }: Props) {
  const [convos, setConvos] = useState(initConvos);
  const [selected, setSelected] = useState<string | null>(initConvos[0]?.id ?? null);
  const [statusFilter, setStatusFilter] = useState('');
  const [topicFilter, setTopicFilter] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
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

  useEffect(() => {
    if (!selected || showNew) { setMsgs([]); return; }
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
    if (evt.type === 'status_changed') { void reloadConvos(); }
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
    if (searchQuery !== '' && !c.subject.toLowerCase().includes(searchQuery.toLowerCase())) { return false; }
    return true;
  });

  const selectedConv = convos.find(c => c.id === selected);
  const selectedTopic = selectedConv ? topics.find(t => t.id === selectedConv.topic_id) : null;
  const canResolve =
    selectedConv !== undefined &&
    selectedConv.status !== 'resolved' &&
    selectedConv.status !== 'closed';

  return (
    <div className="h-full flex flex-col md:flex-row overflow-hidden">
      <div aria-hidden className="absolute opacity-0 pointer-events-none">
        <TurnstileWidget
          key={widgetKey}
          action="chat"
          onSuccess={setToken}
          onExpire={() => setToken(null)}
        />
      </div>

      {/* ── Sidebar ── */}
      <div className={`
        w-full md:w-[300px] lg:w-[320px] flex-shrink-0 flex flex-col overflow-hidden
        bg-white/85 dark:bg-slate-950/90
        backdrop-blur-2xl
        border-r border-slate-200/60 dark:border-white/5
        ${mobileView === 'chat' ? 'hidden md:flex' : 'flex'}
      `}>
        {/* Header */}
        <div className="px-4 pt-5 pb-3.5 border-b border-slate-100/80 dark:border-white/5">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="relative shrink-0">
                <div className="w-10 h-10 rounded-2xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] flex items-center justify-center shadow-lg shadow-[#7645d9]/30">
                  <Headset className="w-5 h-5 text-white" />
                </div>
                <span className="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full bg-emerald-400 border-2 border-white dark:border-slate-950 shadow-sm shadow-emerald-400/50" />
              </div>
              <div>
                <h2 className="font-bold text-[13px] tracking-tight">Support Center</h2>
                <p className="text-[10px] text-muted-foreground/55 mt-0.5">Usually replies in minutes</p>
              </div>
            </div>
            {convos.length > 0 && (
              <span className="text-[10px] font-bold bg-[#7645d9]/10 text-[#7645d9] dark:text-violet-400 px-2 py-0.5 rounded-full border border-[#7645d9]/15">
                {convos.length}
              </span>
            )}
          </div>

          {/* Search */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground/35 pointer-events-none" />
            <input
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              placeholder="Search conversations..."
              className="w-full pl-9 pr-3 py-2 text-[11px] bg-slate-100/70 dark:bg-white/5 border border-slate-200/80 dark:border-white/8 rounded-xl text-foreground placeholder:text-muted-foreground/35 focus:outline-none focus:border-[#7645d9]/40 focus:ring-2 focus:ring-[#7645d9]/8 transition-all"
            />
          </div>
        </div>

        {/* Filters */}
        <div className="flex gap-1.5 px-3 py-2 border-b border-slate-100/80 dark:border-white/5">
          <select
            value={statusFilter}
            onChange={e => setStatusFilter(e.target.value)}
            className="flex-1 px-2 py-1.5 text-[11px] font-medium bg-slate-100/70 dark:bg-white/5 border border-slate-200/80 dark:border-white/8 rounded-lg text-foreground focus:outline-none focus:border-[#7645d9]/40 transition-all cursor-pointer"
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
            className="flex-1 px-2 py-1.5 text-[11px] font-medium bg-slate-100/70 dark:bg-white/5 border border-slate-200/80 dark:border-white/8 rounded-lg text-foreground focus:outline-none focus:border-[#7645d9]/40 transition-all cursor-pointer"
          >
            <option value="">All Topics</option>
            {topics.map(t => (
              <option key={t.id} value={t.id}>{t.label}</option>
            ))}
          </select>
        </div>

        {/* Conversation Cards */}
        <div className="flex-1 overflow-y-auto scrollbar-thin">
          {filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full py-12 text-center px-6">
              <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-[#7645d9]/8 to-[#1fc7d4]/8 border border-[#7645d9]/12 flex items-center justify-center mb-3">
                <Inbox className="w-5 h-5 text-[#7645d9]/40" />
              </div>
              <p className="text-xs font-semibold text-foreground/50 mb-1">No conversations</p>
              <p className="text-[10px] text-muted-foreground/35">Start a new one below</p>
            </div>
          ) : (
            filtered.map(c => {
              const topic = topics.find(t => t.id === c.topic_id);
              const isSel = selected === c.id && !showNew;
              const unread = c.unread_user > 0;
              return (
                <button
                  key={c.id}
                  onClick={() => { setSelected(c.id); setShowNew(false); setMobileView('chat'); }}
                  className={`
                    relative w-full text-left px-4 py-3.5 transition-all
                    border-b border-slate-100/60 dark:border-white/[0.03]
                    ${isSel
                      ? 'bg-gradient-to-r from-[#7645d9]/10 to-[#1fc7d4]/5 border-l-[3px] border-l-[#7645d9] !pl-[13px]'
                      : 'hover:bg-slate-50/80 dark:hover:bg-white/[0.03] border-l-[3px] border-l-transparent'
                    }
                  `}
                >
                  <div className="flex items-start justify-between gap-2 mb-1.5">
                    <p className={`text-[12.5px] leading-snug line-clamp-1 ${unread ? 'font-bold' : 'font-medium text-foreground/70'}`}>
                      {c.subject}
                    </p>
                    <div className="flex items-center gap-1.5 shrink-0 mt-0.5">
                      {unread && (
                        <span className="min-w-[16px] h-4 px-1 rounded-full bg-gradient-to-r from-[#7645d9] to-[#1fc7d4] text-white text-[8px] font-black flex items-center justify-center shadow-sm shadow-[#7645d9]/25">
                          {c.unread_user > 9 ? '9+' : c.unread_user}
                        </span>
                      )}
                      <span className="text-[10px] text-muted-foreground/35 font-medium">{timeAgo(c.last_message_at)}</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <ChatStatusBadge status={c.status} />
                    {topic !== undefined && (
                      <span className="text-[10px] font-semibold text-[#1fc7d4]/75">{topic.label}</span>
                    )}
                  </div>
                </button>
              );
            })
          )}
        </div>

        {/* New Conversation */}
        <div className="p-3 border-t border-slate-100/80 dark:border-white/5">
          <button
            onClick={() => { setShowNew(true); setSelected(null); setMobileView('chat'); }}
            className="w-full py-2.5 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white font-semibold text-[12px] tracking-wide hover:from-[#8755e8] hover:to-[#6840cc] transition-all shadow-lg shadow-[#7645d9]/20 flex items-center justify-center gap-1.5 active:scale-[0.98]"
          >
            <Plus className="w-3.5 h-3.5" />
            New Conversation
          </button>
        </div>
      </div>

      {/* ── Main Chat Panel ── */}
      <div className={`
        flex-1 min-w-0 flex flex-col overflow-hidden
        bg-slate-50/50 dark:bg-slate-900/50
        ${mobileView === 'list' ? 'hidden md:flex' : 'flex'}
      `}>
        {showNew ? (
          <div className="flex-1 flex flex-col overflow-hidden">
            <button
              onClick={handleMobileBack}
              className="md:hidden flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground px-5 pt-4 w-fit transition-colors"
            >
              <ArrowLeft className="w-3.5 h-3.5" /> Back
            </button>
            <ChatTopicSelector topics={topics} onSelect={handleCreate} turnstileToken={token} />
          </div>
        ) : selectedConv !== undefined ? (
          <>
            {/* Chat Header */}
            <div className="flex-shrink-0 bg-white/90 dark:bg-slate-900/90 backdrop-blur-xl border-b border-slate-200/60 dark:border-white/5 shadow-sm shadow-black/[0.03]">
              <div className="h-[3px] bg-gradient-to-r from-[#7645d9] via-[#1fc7d4] to-[#7645d9] bg-[length:200%] animate-[gradient-x_4s_ease_infinite]" />
              <div className="flex items-center gap-3 px-5 py-3.5">
                <button
                  onClick={handleMobileBack}
                  className="md:hidden w-8 h-8 rounded-xl bg-slate-100 dark:bg-white/5 hover:bg-slate-200 dark:hover:bg-white/10 flex items-center justify-center transition-colors shrink-0"
                >
                  <ArrowLeft className="w-4 h-4 text-muted-foreground" />
                </button>
                <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-[#7645d9]/15 to-[#1fc7d4]/15 border border-[#7645d9]/20 flex items-center justify-center shrink-0">
                  <MessageCircle className="w-4 h-4 text-[#7645d9]" />
                </div>
                <div className="flex-1 min-w-0">
                  <h2 className="font-bold text-sm truncate">{selectedConv.subject}</h2>
                  {selectedTopic !== undefined && selectedTopic !== null && (
                    <div className="flex items-center gap-1 mt-0.5">
                      <Tag className="w-2.5 h-2.5 text-[#1fc7d4]/70" />
                      <span className="text-[10px] font-semibold text-[#1fc7d4]/80">{selectedTopic.label}</span>
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  <ChatStatusBadge status={selectedConv.status} />
                  {canResolve && (
                    <button
                      onClick={handleResolve}
                      disabled={isPending}
                      className="hidden sm:flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-500 border border-emerald-500/25 text-[10px] font-bold uppercase tracking-wider transition-all disabled:opacity-40 active:scale-95"
                    >
                      <CheckCircle className="w-3 h-3" /> Resolve
                    </button>
                  )}
                </div>
              </div>
            </div>

            {/* Messages */}
            {loadingMsgs ? (
              <div className="flex-1 flex items-center justify-center">
                <div className="text-center">
                  <Loader2 className="w-6 h-6 text-[#7645d9] animate-spin mx-auto mb-2" />
                  <p className="text-xs text-muted-foreground/50">Loading messages...</p>
                </div>
              </div>
            ) : (
              <ChatMessageList msgs={msgs} userAddr={userAddr} />
            )}

            {/* Mobile resolve */}
            {canResolve && (
              <div className="sm:hidden px-4 py-2 bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm border-t border-slate-200/60 dark:border-white/5">
                <button
                  onClick={handleResolve}
                  disabled={isPending}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-500 border border-emerald-500/25 text-[10px] font-bold uppercase tracking-wider transition-all disabled:opacity-40"
                >
                  <CheckCircle className="w-3 h-3" /> Mark as Resolved
                </button>
              </div>
            )}

            {/* Input */}
            <ChatInput
              onSend={handleSend}
              onUpload={handleUpload}
              onTyping={handleTyping}
              disabled={isPending || selectedConv.status === 'closed'}
              placeholder={selectedConv.status === 'closed' ? 'This conversation is closed' : 'Type your reply...'}
              turnstileToken={token}
              onTokenUsed={handleTokenUsed}
            />
          </>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center px-8">
            <div className="relative mb-5">
              <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-[#7645d9]/10 to-[#1fc7d4]/10 border border-[#7645d9]/15 flex items-center justify-center shadow-sm">
                <MessageCircle className="w-8 h-8 text-[#7645d9]/25" />
              </div>
              <div className="absolute -top-1 -right-1 w-5 h-5 rounded-full bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] flex items-center justify-center shadow-md shadow-[#7645d9]/30">
                <Headset className="w-3 h-3 text-white" />
              </div>
            </div>
            <p className="text-sm font-semibold text-foreground/55 mb-1.5">Select a conversation</p>
            <p className="text-[11px] text-muted-foreground/35 max-w-[180px] leading-relaxed">Choose from the sidebar or start a new conversation</p>
          </div>
        )}
      </div>
    </div>
  );
}
