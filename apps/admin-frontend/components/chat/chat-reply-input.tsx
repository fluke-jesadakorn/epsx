'use client';

import { useState, useTransition, useEffect, useRef } from 'react';
import { Send, CheckCircle, XCircle, UserPlus, Search, ChevronDown, Loader2 } from 'lucide-react';
import { Popover, PopoverContent, PopoverTrigger } from '@/shared/components/ui/popover';
import { listAdminAgents } from '@/app/actions/chat';
import type { AdminAgent } from '@/app/actions/chat';

interface Props {
  onSend: (content: string) => Promise<void>;
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

export function ChatReplyInput({ onSend, onResolve, onClose, onAssign, myWallet, assignedAgent, disabled }: Props) {
  const [msg, setMsg] = useState('');
  const [isPending, startTransition] = useTransition();
  const [open, setOpen] = useState(false);
  const [agents, setAgents] = useState<AdminAgent[]>([]);
  const [search, setSearch] = useState('');
  const [loadingAgents, setLoadingAgents] = useState(false);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    setLoadingAgents(true);
    void (async () => {
      const res = await listAdminAgents(search !== '' ? search : undefined);
      if (Array.isArray(res.data)) {
        setAgents(res.data);
      }
      setLoadingAgents(false);
    })();
  }, [open, search]);

  useEffect(() => {
    if (open) {
      setTimeout(() => searchRef.current?.focus(), 100);
    } else {
      setSearch('');
    }
  }, [open]);

  const handleSend = () => {
    if (msg.trim() === '' || isPending || (disabled ?? false)) {
      return;
    }
    startTransition(() => {
      void (async () => {
        await onSend(msg);
        setMsg('');
      })();
    });
  };

  const handleResolve = () => {
    if (isPending || (disabled ?? false) || onResolve === undefined) {
      return;
    }
    startTransition(() => {
      void onResolve();
    });
  };

  const handleClose = () => {
    if (isPending || (disabled ?? false) || onClose === undefined) {
      return;
    }
    startTransition(() => {
      void onClose();
    });
  };

  const handleAssign = (addr: string) => {
    if (isPending || (disabled ?? false) || onAssign === undefined) {
      return;
    }
    setOpen(false);
    startTransition(() => {
      void onAssign(addr);
    });
  };

  const canSend = msg.trim() !== '' && !isPending && !(disabled ?? false);

  return (
    <div className="border-t border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-4">
      {/* Action buttons */}
      <div className="flex gap-2 mb-3">
        {onAssign && (
          <Popover open={open} onOpenChange={setOpen}>
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
            <PopoverContent align="start" className="w-72 p-0 bg-white dark:bg-slate-900 border-gray-200 dark:border-slate-700">
              {/* Search */}
              <div className="p-2 border-b border-gray-200 dark:border-slate-700">
                <div className="flex items-center gap-2 px-2 py-1.5 bg-gray-100 dark:bg-slate-800/60 rounded-lg">
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

              {/* Assign to me (quick option) */}
              {myWallet && myWallet !== (assignedAgent ?? '') && (
                <div className="p-1 border-b border-gray-200 dark:border-slate-700">
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

              {/* Agent list */}
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
                          <div className="w-6 h-6 rounded-full bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700 flex items-center justify-center shrink-0">
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

      {/* Input area */}
      <div className="flex gap-2">
        <div className="flex-1 bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700 rounded-xl focus-within:border-violet-500/40 focus-within:ring-1 focus-within:ring-violet-500/20 transition-all overflow-hidden">
          <textarea
            value={msg}
            onChange={e => setMsg(e.target.value)}
            onKeyDown={e => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder="Type your reply..."
            disabled={isPending || disabled}
            className="w-full px-4 py-3 bg-transparent text-sm text-foreground placeholder:text-muted-foreground/40 focus:outline-none resize-none disabled:opacity-40 disabled:cursor-not-allowed"
            rows={2}
          />
        </div>
        <button
          onClick={handleSend}
          disabled={!canSend}
          className={`px-4 rounded-xl flex items-center justify-center transition-all ${
            canSend
              ? 'bg-gradient-to-r from-violet-500 to-purple-600 text-white hover:from-violet-400 hover:to-purple-500 shadow-sm shadow-violet-500/20'
              : 'bg-gray-100 dark:bg-slate-800/40 text-muted-foreground/30 cursor-not-allowed'
          }`}
        >
          <Send className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
