'use client';

import { ChevronLeft, X, CheckCircle2, Headset } from 'lucide-react';
import { ChatStatusBadge } from './chat-status-badge';

interface HeaderProps {
  subject: string;
  status: 'open' | 'in_progress' | 'resolved' | 'closed';
  onBack?: () => void;
  onClose?: () => void;
  onResolve?: () => void;
  showResolve?: boolean;
}

export function ChatHeader({ subject, status, onBack, onClose, onResolve, showResolve = true }: HeaderProps) {
  const canResolve = showResolve && status !== 'resolved' && status !== 'closed';

  return (
    <div className="flex items-center justify-between px-4 py-3 border-b border-slate-200 dark:border-white/5 bg-slate-50 dark:bg-slate-900/60">
      <div className="flex items-center gap-2 flex-1 min-w-0">
        {onBack && (
          <button
            onClick={onBack}
            className="shrink-0 w-8 h-8 rounded-lg hover:bg-muted flex items-center justify-center transition-colors"
            aria-label="Go back"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
        )}
        <div className="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center shrink-0">
          <Headset className="w-4 h-4 text-blue-400" />
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-sm truncate leading-tight">{subject}</h3>
          <div className="mt-0.5">
            <ChatStatusBadge status={status} />
          </div>
        </div>
      </div>
      <div className="flex items-center gap-1.5 shrink-0">
        {canResolve && onResolve && (
          <button
            onClick={onResolve}
            className="shrink-0 h-7 px-2.5 rounded-lg bg-emerald-500/10 hover:bg-emerald-500/20 flex items-center gap-1.5 transition-colors text-emerald-500 text-xs font-medium border border-emerald-500/20"
            aria-label="Mark as resolved"
          >
            <CheckCircle2 className="w-3.5 h-3.5" />
            Resolve
          </button>
        )}
        {onClose && (
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-lg hover:bg-muted flex items-center justify-center transition-colors"
            aria-label="Close chat"
          >
            <X className="w-4 h-4 text-muted-foreground" />
          </button>
        )}
      </div>
    </div>
  );
}
