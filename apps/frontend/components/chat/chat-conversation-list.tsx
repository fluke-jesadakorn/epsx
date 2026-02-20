'use client';

import type { ChatConversation } from '@/shared/api/chat';
import { ChatStatusBadge } from './chat-status-badge';
import { formatDistanceToNow } from 'date-fns';
import { Plus, MessageCircle, ChevronRight } from 'lucide-react';

interface ConvoListProps {
  convos: ChatConversation[];
  onSelect: (id: string) => void;
  onNew: () => void;
}

export function ChatConversationList({ convos, onSelect, onNew }: ConvoListProps) {
  if (convos.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-8 text-center">
        <div className="w-16 h-16 rounded-2xl bg-blue-500/10 flex items-center justify-center mb-5">
          <MessageCircle className="w-8 h-8 text-blue-400" />
        </div>
        <p className="text-base font-semibold mb-1">No conversations yet</p>
        <p className="text-xs text-muted-foreground mb-6 max-w-[200px]">
          Start a new chat to get help from our support team
        </p>
        <button
          onClick={onNew}
          className="px-5 py-2.5 rounded-xl bg-gradient-to-r from-blue-500 to-blue-600 text-white font-medium text-sm hover:from-blue-400 hover:to-blue-500 transition-all shadow-sm shadow-blue-500/20 flex items-center gap-2"
        >
          <Plus className="w-4 h-4" />
          New Conversation
        </button>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col">
      <div className="flex items-center justify-between px-4 py-3 border-b border-slate-200 dark:border-slate-700">
        <span className="text-xs text-muted-foreground font-medium">
          {convos.length} conversation{convos.length !== 1 ? 's' : ''}
        </span>
        <button
          onClick={onNew}
          className="h-7 px-3 rounded-lg bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 transition-colors flex items-center gap-1.5 text-xs font-semibold border border-blue-500/20"
          aria-label="New conversation"
        >
          <Plus className="w-3.5 h-3.5" />
          New
        </button>
      </div>
      <div className="flex-1 overflow-y-auto">
        {convos.map((convo) => {
          const timeAgo = formatDistanceToNow(new Date(convo.last_message_at), { addSuffix: true });
          const hasUnread = convo.unread_user > 0;
          return (
            <button
              key={convo.id}
              onClick={() => onSelect(convo.id)}
              className={`group w-full px-4 py-3.5 border-b border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-gray-100 dark:bg-slate-800/30 transition-all text-left ${
                hasUnread ? 'bg-blue-50 dark:bg-blue-500/5 border-l-2 border-l-blue-500' : ''
              }`}
            >
              <div className="flex items-start justify-between gap-2 mb-1.5">
                <h4 className={`text-sm truncate flex-1 leading-tight ${hasUnread ? 'font-bold text-foreground' : 'font-medium text-foreground/80'}`}>
                  {convo.subject}
                </h4>
                <ChevronRight className="w-4 h-4 text-muted-foreground/30 shrink-0 mt-0.5 group-hover:text-muted-foreground/60 group-hover:translate-x-0.5 transition-all" />
              </div>
              <div className="flex items-center gap-2">
                <ChatStatusBadge status={convo.status} />
                <span className="text-[10px] text-muted-foreground/60">{timeAgo}</span>
                {hasUnread && (
                  <span className="ml-auto shrink-0 min-w-[20px] h-5 px-1.5 rounded-full bg-blue-600 text-white text-[10px] flex items-center justify-center font-bold shadow-sm shadow-blue-500/20">
                    {convo.unread_user > 9 ? '9+' : convo.unread_user}
                  </span>
                )}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
