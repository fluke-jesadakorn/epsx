/* eslint-disable max-lines-per-function */
'use client';

import type { AdminAgent } from '@/app/actions/chat';
import { listAdminAgents } from '@/app/actions/chat';
import { TurnstileWidget } from '@/shared/components/turnstile-widget';
import { Popover, PopoverContent, PopoverTrigger } from '@/shared/components/ui/popover';
import { CheckCircle, ChevronDown, Loader2, MessageSquare, Paperclip, Search, Send, UserPlus, X, XCircle } from 'lucide-react';
import { useCallback, useEffect, useRef, useState, useTransition } from 'react';

// ============================================================================
// CANNED RESPONSES (stored in localStorage for v1)
// ============================================================================

const DEFAULT_RESPONSES = [
  { id: '1', label: 'Welcome', text: "Hello! Thanks for reaching out to EPSX support. How can I help you today?" },
  { id: '2', label: 'Looking into it', text: "I'm looking into this for you right now. I'll get back to you shortly." },
  { id: '3', label: 'Need more info', text: "Could you please provide more details? Specifically, which wallet address and what transaction hash are you referring to?" },
  { id: '4', label: 'Resolved', text: "I'm glad we could resolve this for you! Is there anything else I can help with?" },
  { id: '5', label: 'Docs link', text: "You can find more information in our documentation at https://epsx.io/docs. Let me know if you have any questions!" },
];

interface Props {
  onSend: (content: string, turnstileToken: string) => Promise<void>;
  onUpload?: (file: File) => Promise<void>;
  onTyping?: (isTyping: boolean) => void;
  onResolve?: () => Promise<void>;
  onClose?: () => Promise<void>;
  onAssign?: (walletAddress: string) => Promise<void>;
  myWallet?: string;
  assignedAgent?: string | null;
  disabled?: boolean;
}

function truncAddr(addr: string): string {
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
}

export function ChatReplyInput({ onSend, onUpload, onTyping, onResolve, onClose, onAssign, myWallet, assignedAgent, disabled }: Props) {
  const [msg, setMsg] = useState('');
  const [isPending, startTransition] = useTransition();
  const [assignOpen, setAssignOpen] = useState(false);
  const [cannedOpen, setCannedOpen] = useState(false);
  const [pendingFile, setPendingFile] = useState<File | null>(null);
  const [agents, setAgents] = useState<AdminAgent[]>([]);
  const [search, setSearch] = useState('');
  const [loadingAgents, setLoadingAgents] = useState(false);
  const searchRef = useRef<HTMLInputElement>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const typingTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);
  const [turnstileKey, setTurnstileKey] = useState(0);

  useEffect(() => {
    if (!assignOpen) return;
    setLoadingAgents(true);
    void (async () => {
      const res = await listAdminAgents(search !== '' ? search : undefined);
      if (Array.isArray(res.data)) {
        setAgents(res.data);
      }
      setLoadingAgents(false);
    })();
  }, [assignOpen, search]);

  useEffect(() => {
    if (assignOpen) {
      setTimeout(() => searchRef.current?.focus(), 100);
    } else {
      setSearch('');
    }
  }, [assignOpen]);

  const emitTyping = useCallback((typing: boolean) => {
    if (isTypingRef.current === typing) return;
    isTypingRef.current = typing;
    onTyping?.(typing);
  }, [onTyping]);

  const handleMsgChange = (v: string) => {
    setMsg(v);
    if (v.length > 0) {
      emitTyping(true);
      if (typingTimer.current) clearTimeout(typingTimer.current);
      typingTimer.current = setTimeout(() => emitTyping(false), 2000);
    } else {
      emitTyping(false);
    }
  };

  const handleSend = () => {
    if (pendingFile && onUpload) {
      const f = pendingFile;
      setPendingFile(null);
      setMsg('');
      emitTyping(false);
      startTransition(() => { void onUpload(f); });
      return;
    }
    if (msg.trim() === '' || isPending || (disabled ?? false) || !turnstileToken) return;
    const content = msg;
    const token = turnstileToken;
    startTransition(() => {
      void (async () => {
        await onSend(content, token);
        setMsg('');
        setTurnstileToken(null);
        setTurnstileKey(k => k + 1);
        emitTyping(false);
      })();
    });
  };

  const handleResolve = () => {
    if (isPending || (disabled ?? false) || onResolve === undefined) return;
    startTransition(() => { void onResolve(); });
  };

  const handleClose = () => {
    if (isPending || (disabled ?? false) || onClose === undefined) return;
    startTransition(() => { void onClose(); });
  };

  const handleAssign = (addr: string) => {
    if (isPending || (disabled ?? false) || onAssign === undefined) return;
    setAssignOpen(false);
    startTransition(() => { void onAssign(addr); });
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) setPendingFile(file);
    e.target.value = '';
  };

  const handleCannedSelect = (text: string) => {
    setMsg(text);
    setCannedOpen(false);
    emitTyping(true);
  };

  const canSend = (msg.trim() !== '' || pendingFile !== null) && !isPending && !(disabled ?? false) && (pendingFile !== null || turnstileToken !== null);

  return (
    <div className="border-t border-border/20 bg-card p-4">
      {/* Action buttons row */}
      <div className="flex gap-2 mb-3 flex-wrap">
        {onAssign && (
          <Popover open={assignOpen} onOpenChange={setAssignOpen}>
            <PopoverTrigger asChild>
              <button
                disabled={isPending || disabled}
                className="px-3 py-1.5 text-[11px] font-bold bg-blue-500/10 text-blue-400 border border-blue-500/20 rounded-lg hover:bg-blue-500/20 transition-all disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5 uppercase tracking-wide"
              >
                <UserPlus className="w-3.5 h-3.5" />
                Assign
                <ChevronDown className="w-3 h-3" />
              </button>
            </PopoverTrigger>
            <PopoverContent align="start" className="w-72 p-0 bg-card border-border/20">
              <div className="p-2 border-b border-border/20">
                <div className="flex items-center gap-2 px-2 py-1.5 bg-gray-100 dark:bg-card/60 rounded-lg">
                  <Search className="w-3.5 h-3.5 text-muted-foreground/50" />
                  <input
                    ref={searchRef}
                    value={search}
                    onChange={e => setSearch(e.target.value)}
                    placeholder="Search by wallet..."
                    className="flex-1 bg-transparent text-xs text-foreground placeholder:text-muted-foreground/40 focus:outline-none"
                  />
                </div>
              </div>
              {myWallet && myWallet !== (assignedAgent ?? '') && (
                <div className="p-1 border-b border-border/20">
                  <button
                    onClick={() => handleAssign(myWallet)}
                    className="w-full px-3 py-2 text-left rounded-md hover:bg-blue-500/10 transition-colors flex items-center gap-2"
                  >
                    <div className="w-6 h-6 rounded-full bg-blue-500/15 border border-blue-500/20 flex items-center justify-center">
                      <UserPlus className="w-3 h-3 text-blue-400" />
                    </div>
                    <div>
                      <p className="text-xs font-semibold text-blue-400">Assign to me</p>
                      <p className="text-[10px] text-muted-foreground/50 font-mono">{truncAddr(myWallet)}</p>
                    </div>
                  </button>
                </div>
              )}
              <div className="max-h-48 overflow-y-auto p-1">
                {loadingAgents ? (
                  <div className="flex items-center justify-center py-4">
                    <Loader2 className="w-4 h-4 text-muted-foreground/40 animate-spin" />
                  </div>
                ) : agents.length === 0 ? (
                  <p className="text-xs text-muted-foreground/40 text-center py-4">No users found</p>
                ) : (
                  agents.map(a => {
                    const isMe = a.wallet_address === myWallet;
                    const isAssigned = a.wallet_address === (assignedAgent ?? '');
                    return (
                      <button
                        key={a.wallet_address}
                        onClick={() => handleAssign(a.wallet_address)}
                        disabled={isAssigned}
                        className="w-full px-3 py-2 text-left rounded-md hover:bg-gray-100 dark:hover:bg-slate-800/60 transition-colors flex items-center justify-between gap-2 disabled:opacity-40 disabled:cursor-not-allowed"
                      >
                        <div className="flex items-center gap-2 min-w-0">
                          <div className="w-6 h-6 rounded-full bg-gray-100 dark:bg-card/60 border border-border/20 flex items-center justify-center shrink-0">
                            <span className="text-[9px] font-bold text-muted-foreground/60">
                              {a.wallet_address.slice(2, 4).toUpperCase()}
                            </span>
                          </div>
                          <div className="min-w-0">
                            <p className="text-xs font-mono text-foreground truncate">
                              {truncAddr(a.wallet_address)}
                              {isMe && <span className="text-blue-400 ml-1 font-sans">(me)</span>}
                            </p>
                            <p className="text-[10px] text-muted-foreground/40 capitalize">{a.tier}</p>
                          </div>
                        </div>
                        {isAssigned && (
                          <span className="text-[9px] font-bold text-cyan-400 uppercase">Assigned</span>
                        )}
                      </button>
                    );
                  })
                )}
              </div>
            </PopoverContent>
          </Popover>
        )}

        {/* Canned responses */}
        <Popover open={cannedOpen} onOpenChange={setCannedOpen}>
          <PopoverTrigger asChild>
            <button
              disabled={isPending || disabled}
              className="px-3 py-1.5 text-[11px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 rounded-lg hover:bg-amber-500/20 transition-all disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5 uppercase tracking-wide"
            >
              <MessageSquare className="w-3.5 h-3.5" />
              Saved
            </button>
          </PopoverTrigger>
          <PopoverContent align="start" className="w-80 p-0 bg-card border-border/20">
            <div className="p-2 border-b border-border/20">
              <p className="text-xs font-bold text-muted-foreground uppercase tracking-wide px-1">Saved Replies</p>
            </div>
            <div className="max-h-56 overflow-y-auto p-1">
              {DEFAULT_RESPONSES.map(r => (
                <button
                  key={r.id}
                  onClick={() => handleCannedSelect(r.text)}
                  className="w-full px-3 py-2.5 text-left rounded-md hover:bg-gray-100 dark:hover:bg-slate-800/60 transition-colors"
                >
                  <p className="text-xs font-semibold text-foreground mb-0.5">{r.label}</p>
                  <p className="text-[11px] text-muted-foreground/60 line-clamp-2">{r.text}</p>
                </button>
              ))}
            </div>
          </PopoverContent>
        </Popover>

        {onResolve && (
          <button
            onClick={handleResolve}
            disabled={isPending || disabled}
            className="px-3 py-1.5 text-[11px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-lg hover:bg-emerald-500/20 transition-all disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5 uppercase tracking-wide"
          >
            <CheckCircle className="w-3.5 h-3.5" />
            Resolve
          </button>
        )}
        {onClose && (
          <button
            onClick={handleClose}
            disabled={isPending || disabled}
            className="px-3 py-1.5 text-[11px] font-bold bg-zinc-500/10 text-zinc-400 border border-zinc-500/20 rounded-lg hover:bg-zinc-500/20 transition-all disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5 uppercase tracking-wide"
          >
            <XCircle className="w-3.5 h-3.5" />
            Close
          </button>
        )}
      </div>

      {/* File preview */}
      {pendingFile !== null && (
        <div className="flex items-center gap-2 mb-2 px-3 py-2 bg-violet-500/10 border border-violet-500/20 rounded-xl text-xs text-violet-400">
          <Paperclip className="w-3.5 h-3.5 shrink-0" />
          <span className="flex-1 truncate">{pendingFile.name}</span>
          <button onClick={() => setPendingFile(null)} className="shrink-0 hover:opacity-70">
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Input area */}
      <div className="flex gap-2">
        <div className="flex-1 bg-gray-100 dark:bg-card/60 border border-border/20 rounded-xl focus-within:border-violet-500/40 focus-within:ring-1 focus-within:ring-violet-500/20 transition-all overflow-hidden">
          <div className="flex items-end">
            {onUpload !== undefined && (
              <>
                <input
                  ref={fileRef}
                  type="file"
                  accept="image/jpeg,image/png,image/gif,image/webp,application/pdf"
                  onChange={handleFileChange}
                  className="hidden"
                />
                <button
                  onClick={() => fileRef.current?.click()}
                  disabled={isPending || disabled}
                  className="m-2 w-7 h-7 rounded-lg flex items-center justify-center text-muted-foreground/40 hover:text-muted-foreground hover:bg-gray-200 dark:hover:bg-slate-700 transition-all disabled:opacity-30 shrink-0"
                  aria-label="Attach file"
                >
                  <Paperclip className="w-4 h-4" />
                </button>
              </>
            )}
            <textarea
              value={msg}
              onChange={e => handleMsgChange(e.target.value)}
              onKeyDown={e => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  handleSend();
                }
              }}
              placeholder="Type your reply... (supports **markdown**)"
              disabled={isPending || disabled}
              className="flex-1 px-4 py-3 bg-transparent text-sm text-foreground placeholder:text-muted-foreground/40 focus:outline-none resize-none disabled:opacity-40 disabled:cursor-not-allowed"
              rows={2}
            />
          </div>
        </div>
        <button
          onClick={handleSend}
          disabled={!canSend}
          className={`px-4 rounded-xl flex items-center justify-center transition-all ${canSend
              ? 'bg-gradient-to-r from-violet-500 to-purple-600 text-white hover:from-violet-400 hover:to-purple-500 shadow-sm shadow-violet-500/20'
              : 'bg-gray-100 dark:bg-card/40 text-muted-foreground/30 cursor-not-allowed'
            }`}
        >
          <Send className="w-4 h-4" />
        </button>
      </div>

      <TurnstileWidget
        key={turnstileKey}
        action="admin_chat_reply"
        onSuccess={setTurnstileToken}
        onExpire={() => setTurnstileToken(null)}
        className="mt-3 flex justify-start"
      />
    </div>
  );
}
