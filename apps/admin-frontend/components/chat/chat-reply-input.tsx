'use client';

import { useState, useTransition } from 'react';
import { Send, CheckCircle, XCircle, UserPlus } from 'lucide-react';

interface Props {
  onSend: (content: string) => Promise<void>;
  onResolve?: () => Promise<void>;
  onClose?: () => Promise<void>;
  onAssignMe?: () => Promise<void>;
  disabled?: boolean;
}

export function ChatReplyInput({ onSend, onResolve, onClose, onAssignMe, disabled }: Props) {
  const [msg, setMsg] = useState('');
  const [isPending, startTransition] = useTransition();

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

  const handleAssign = () => {
    if (isPending || (disabled ?? false) || onAssignMe === undefined) {
      return;
    }
    startTransition(() => {
      void onAssignMe();
    });
  };

  const canSend = msg.trim() !== '' && !isPending && !(disabled ?? false);

  return (
    <div className="border-t border-gray-200 dark:border-border bg-white dark:bg-card p-4">
      {/* Action buttons */}
      <div className="flex gap-2 mb-3">
        {onAssignMe && (
          <button
            onClick={handleAssign}
            disabled={isPending || disabled}
            className="px-3 py-1.5 text-[11px] font-bold bg-blue-500/10 text-blue-400 border border-blue-500/20 rounded-lg hover:bg-blue-500/20 transition-all disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5 uppercase tracking-wide"
          >
            <UserPlus className="w-3.5 h-3.5" />
            Assign
          </button>
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
        <div className="flex-1 bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-border rounded-xl focus-within:border-violet-500/40 focus-within:ring-1 focus-within:ring-violet-500/20 transition-all overflow-hidden">
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
