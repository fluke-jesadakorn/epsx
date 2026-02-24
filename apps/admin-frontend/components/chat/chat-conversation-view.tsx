/* eslint-disable max-lines-per-function */
'use client';

import { assignAgent, getMessages, markAsRead, sendReply, updateStatus } from '@/app/actions/chat';
import type { ChatAttachment, ChatConversation, ChatMessage, ChatTopic } from '@/shared/api/chat';
import { COOKIES } from '@/shared/auth/cookies';
import { useSharedAuth } from '@/shared/components/auth';
import { ChatMarkdown } from '@/shared/components/chat/chat-markdown';
import type { ChatSSEEvent } from '@/shared/hooks/use-chat-sse';
import { useChatSSE } from '@/shared/hooks/use-chat-sse';
import { ArrowLeft, Bot, Check, CheckCheck, Download, FileText, Headset, MessageCircle, Tag, User, UserCheck, Wallet, X, ZoomIn, ZoomOut } from 'lucide-react';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { ChatReplyInput } from './chat-reply-input';
import { ChatStatusBadge } from './chat-status-badge';

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
  return new Date(date).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
}

function formatDate(date: string): string {
  return new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

function getToken(): string | null {
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
    return user.access ?? null;
  } catch { return null; }
}

function getBackendUrl(): string {
  if (typeof window !== 'undefined') {
    try {
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const { getBackendUrl: gbu } = require('@/shared/utils/url-resolver') as { getBackendUrl: (ctx: string) => string };
      return gbu('client');
    } catch { /* ignore */ }
  }
  return '';
}

async function uploadFile(convId: string, file: File): Promise<ChatMessage | null> {
  const token = getToken();
  const form = new FormData();
  form.append('file', file);
  try {
    const res = await fetch(`${getBackendUrl()}/api/admin/chat/conversations/${convId}/upload`, {
      method: 'POST',
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      body: form,
    });
    const json = await res.json() as { success?: boolean; data?: { message?: ChatMessage } };
    return json.success === true ? (json.data?.message ?? null) : null;
  } catch { return null; }
}

async function notifyTyping(convId: string, isTyping: boolean): Promise<void> {
  const token = getToken();
  try {
    await fetch(`${getBackendUrl()}/api/admin/chat/conversations/${convId}/typing`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...(token ? { Authorization: `Bearer ${token}` } : {}) },
      body: JSON.stringify({ is_typing: isTyping }),
    });
  } catch { /* non-critical */ }
}


function ImageLightbox({ src, alt, onClose }: { src: string; alt: string; onClose: () => void }) {
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const dragStart = useRef({ x: 0, y: 0 });

  const zoomIn = useCallback(() => setZoom(z => Math.min(z + 0.5, 4)), []);
  const zoomOut = useCallback(() => setZoom(z => {
    const next = Math.max(z - 0.5, 0.5);
    if (next <= 1) setPan({ x: 0, y: 0 });
    return next;
  }), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      if (e.key === '+' || e.key === '=') zoomIn();
      if (e.key === '-') zoomOut();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose, zoomIn, zoomOut]);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.stopPropagation();
    setZoom(z => {
      const next = Math.max(0.5, Math.min(4, z * (1 - e.deltaY * 0.001)));
      if (next <= 1) setPan({ x: 0, y: 0 });
      return next;
    });
  }, []);

  const handlePointerDown = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    if (zoom <= 1) return;
    setIsDragging(true);
    dragStart.current = { x: e.clientX - pan.x, y: e.clientY - pan.y };
    e.currentTarget.setPointerCapture(e.pointerId);
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    if (!isDragging) return;
    setPan({ x: e.clientX - dragStart.current.x, y: e.clientY - dragStart.current.y });
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    setIsDragging(false);
    e.currentTarget.releasePointerCapture(e.pointerId);
  };

  return createPortal(
    <div
      className="fixed inset-0 z-[9999] bg-black/85 backdrop-blur-sm flex items-center justify-center overflow-hidden"
      onClick={onClose}
      onWheel={handleWheel}
    >
      <div className="absolute top-4 right-4 flex items-center gap-1.5 z-10">
        <button onClick={e => { e.stopPropagation(); zoomIn(); }} className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors">
          <ZoomIn className="w-4 h-4" />
        </button>
        <button onClick={e => { e.stopPropagation(); zoomOut(); }} className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors">
          <ZoomOut className="w-4 h-4" />
        </button>
        <button onClick={e => { e.stopPropagation(); onClose(); }} className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors ml-1">
          <X className="w-4 h-4" />
        </button>
      </div>
      <div className="absolute bottom-4 inset-x-0 text-center pointer-events-none z-10">
        <span className="text-white/40 text-xs px-3 py-1.5 rounded-full bg-black/50 backdrop-blur-md">
          {Math.round(zoom * 100)}% · scroll to zoom · drag to pan · esc to close
        </span>
      </div>
      <div className="w-full h-full flex items-center justify-center">
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={src}
          alt={alt}
          draggable={false}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerUp}
          onClick={e => e.stopPropagation()}
          style={{
            transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
            transition: isDragging ? 'none' : 'transform 0.1s ease-out',
            cursor: zoom > 1 ? (isDragging ? 'grabbing' : 'grab') : 'default',
            touchAction: 'none'
          }}
          className="max-w-[90vw] max-h-[90vh] object-contain rounded-xl select-none"
        />
      </div>
    </div>,
    document.body
  );
}

function AttachmentView({ att, isRight }: { att: ChatAttachment; isRight: boolean }) {
  const [open, setOpen] = useState(false);
  const isImage = att.file_type.startsWith('image/');
  const src = att.url;

  if (isImage) {
    return (
      <>
        <button type="button" onClick={() => setOpen(true)} className="block mt-2 cursor-zoom-in focus:outline-none">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img src={src} alt={att.filename} className="max-w-full max-h-48 rounded-xl border border-white/10 object-cover hover:opacity-90 transition-opacity" />
        </button>
        {open && <ImageLightbox src={src} alt={att.filename} onClose={() => setOpen(false)} />}
      </>
    );
  }

  return (
    <a
      href={src}
      target="_blank"
      rel="noopener noreferrer"
      download={att.filename}
      className={`flex items-center gap-2 mt-2 px-3 py-2 rounded-xl border transition-colors ${isRight
        ? 'bg-violet-500/10 border-violet-500/20 hover:bg-violet-500/20'
        : 'bg-gray-200 dark:bg-slate-700/50 border-gray-300 dark:border-slate-600 hover:bg-gray-300 dark:hover:bg-slate-700'
        }`}
    >
      <FileText className="w-4 h-4 shrink-0 text-muted-foreground" />
      <div className="min-w-0 flex-1">
        <p className="text-xs font-medium truncate">{att.filename}</p>
        <p className="text-[10px] text-muted-foreground/60">{(att.size / 1024).toFixed(1)} KB</p>
      </div>
      <Download className="w-3.5 h-3.5 shrink-0 text-muted-foreground/60" />
    </a>
  );
}

export function ChatConversationView({ conv, topics, onUpdate, onBack }: Props) {
  const [msgs, setMsgs] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(true);
  const [userTyping, setUserTyping] = useState(false);
  const [readUpToId, setReadUpToId] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const markedRef = useRef<string | null>(null);
  const typingTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
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
  }, [msgs, userTyping]);

  const handleSSE = useCallback((evt: ChatSSEEvent) => {
    if (evt.type === 'new_message' && evt.message && evt.conversation_id === conv.id) {
      setMsgs((prev) => prev.some((m) => m.id === evt.message?.id) ? prev : [...prev, evt.message as ChatMessage]);
      setUserTyping(false);
      void markAsRead(conv.id);
    }
    if ((evt.type === 'typing_start' || evt.type === 'typing_stop') && evt.conversation_id === conv.id && evt.sender === 'user') {
      const typing = evt.type === 'typing_start';
      setUserTyping(typing);
      if (typing) {
        if (typingTimer.current) clearTimeout(typingTimer.current);
        typingTimer.current = setTimeout(() => setUserTyping(false), 5000);
      }
    }
    if (evt.type === 'messages_read' && evt.conversation_id === conv.id && evt.reader === 'user') {
      setMsgs((prev) => {
        const lastAgentMsg = [...prev].reverse().find((m) => m.sender_type === 'agent');
        if (lastAgentMsg) setReadUpToId(lastAgentMsg.id);
        return prev;
      });
    }
    if ((evt.type === 'status_changed' || evt.type === 'agent_assigned') && evt.conversation_id === conv.id) {
      onUpdate();
    }
  }, [conv.id, onUpdate]);

  useChatSSE({ enabled: true, mode: 'admin', onEvent: handleSSE });

  const handleSend = async (content: string, turnstileToken: string) => {
    const res = await sendReply(conv.id, content, turnstileToken);
    if (res.success && res.data) {
      setMsgs(prev => [...prev, res.data as ChatMessage]);
      onUpdate();
    }
  };

  const handleUpload = async (file: File) => {
    const msg = await uploadFile(conv.id, file);
    if (msg) {
      setMsgs(prev => [...prev, msg]);
      onUpdate();
    }
  };

  const handleTyping = (isTyping: boolean) => {
    void notifyTyping(conv.id, isTyping);
  };

  const handleResolve = async () => { await updateStatus(conv.id, 'resolved'); onUpdate(); };
  const handleClose = async () => { await updateStatus(conv.id, 'closed'); onUpdate(); };
  const handleAssign = async (addr: string) => { await assignAgent(conv.id, addr); onUpdate(); };

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
      <div ref={scrollRef} className="flex-1 overflow-y-auto p-5 space-y-4 bg-gray-50 dark:bg-slate-950/20">
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
            const attachments = m.metadata?.attachments ?? [];
            const isAttachmentOnly = m.content.startsWith('[attachment:') && attachments.length > 0;
            const showDate = i === 0 || formatDate(msgs[i - 1]?.created_at ?? '') !== formatDate(m.created_at);

            // Read receipt for agent messages
            let readReached = false;
            if (!readReached && readUpToId && m.id === readUpToId) readReached = true;
            const isRead = isRight && (readReached || m.id === readUpToId || m.is_read);

            if (isSystem) {
              return (
                <div key={m.id}>
                  {showDate && <DateSep date={m.created_at} />}
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
                {showDate && <DateSep date={m.created_at} />}
                <div className={`flex gap-2.5 ${isRight ? 'justify-end' : 'justify-start'}`}>
                  {!isRight && (
                    <div className="shrink-0 w-8 h-8 rounded-full bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700 flex items-center justify-center mt-5">
                      <User className="w-4 h-4 text-muted-foreground/60" />
                    </div>
                  )}
                  <div className={`max-w-[70%] flex flex-col ${isRight ? 'items-end' : 'items-start'}`}>
                    <div className="flex items-center gap-1.5 mb-1 px-1">
                      {isAi ? <Bot className="w-3 h-3 text-purple-400" /> : isAgent ? <Headset className="w-3 h-3 text-cyan-400" /> : null}
                      <span className="text-[10px] font-bold text-muted-foreground/60 uppercase tracking-wider">
                        {isAi ? 'AI' : isAgent ? 'Agent' : 'User'}
                      </span>
                      <span className="text-[10px] text-muted-foreground/30">{formatTime(m.created_at)}</span>
                    </div>
                    <div className={`rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${isRight
                      ? isAi
                        ? 'bg-purple-500/10 border border-purple-500/20 text-foreground rounded-br-md'
                        : 'bg-gradient-to-br from-violet-500/15 to-purple-500/10 border border-violet-500/20 text-foreground rounded-br-md'
                      : 'bg-gray-100 dark:bg-slate-800/50 border border-gray-200 dark:border-slate-700 text-foreground rounded-bl-md'
                      }`}>
                      {!isAttachmentOnly && (
                        <ChatMarkdown text={m.content} />
                      )}
                      {attachments.map((att, ai) => (
                        <AttachmentView key={ai} att={att} isRight={isRight} />
                      ))}
                    </div>
                    {/* Read receipt for agent messages */}
                    {isRight && (
                      <div className="flex items-center gap-1 mt-1 px-1">
                        {isRead
                          ? <CheckCheck className="w-3.5 h-3.5 text-blue-400" />
                          : <Check className="w-3.5 h-3.5 text-muted-foreground/30" />
                        }
                      </div>
                    )}
                  </div>
                  {isRight && (
                    <div className={`shrink-0 w-8 h-8 rounded-full flex items-center justify-center mt-5 ${isAi ? 'bg-purple-500/15 border border-purple-500/20' : 'bg-cyan-500/15 border border-cyan-500/20'
                      }`}>
                      {isAi ? <Bot className="w-4 h-4 text-purple-400" /> : <Headset className="w-4 h-4 text-cyan-400" />}
                    </div>
                  )}
                </div>
              </div>
            );
          })
        )}

        {/* User typing indicator */}
        {userTyping && (
          <div className="flex gap-2.5 justify-start">
            <div className="shrink-0 w-8 h-8 rounded-full bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700 flex items-center justify-center">
              <User className="w-4 h-4 text-muted-foreground/60" />
            </div>
            <div className="bg-gray-100 dark:bg-slate-800/50 border border-gray-200 dark:border-slate-700 rounded-2xl rounded-bl-md px-4 py-3 flex items-center gap-1.5">
              <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/50 animate-bounce" style={{ animationDelay: '0ms' }} />
              <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/50 animate-bounce" style={{ animationDelay: '150ms' }} />
              <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/50 animate-bounce" style={{ animationDelay: '300ms' }} />
            </div>
          </div>
        )}
      </div>

      {/* Reply Input */}
      <ChatReplyInput
        onSend={handleSend}
        onUpload={handleUpload}
        onTyping={handleTyping}
        onResolve={canResolve ? handleResolve : undefined}
        onClose={canClose ? handleClose : undefined}
        onAssign={handleAssign}
        myWallet={(walletAddress as string | undefined) ?? ''}
        assignedAgent={conv.assigned_agent}
      />
    </div>
  );
}

function DateSep({ date }: { date: string }) {
  return (
    <div className="flex items-center gap-3 my-4">
      <div className="flex-1 h-px bg-white dark:bg-white/[0.04]" />
      <span className="text-[10px] text-muted-foreground/40 font-medium uppercase tracking-widest">
        {new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
      </span>
      <div className="flex-1 h-px bg-white dark:bg-white/[0.04]" />
    </div>
  );
}
