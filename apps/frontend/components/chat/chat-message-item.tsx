'use client';

import type { ChatMessage } from '@/shared/api/chat';
import { formatDistanceToNow } from 'date-fns';
import { Bot, Headset, Info } from 'lucide-react';

interface MsgItemProps {
  msg: ChatMessage;
  isUser: boolean;
}

export function ChatMessageItem({ msg, isUser }: MsgItemProps) {
  const timestamp = formatDistanceToNow(new Date(msg.created_at), { addSuffix: true });

  if (msg.sender_type === 'system') {
    return (
      <div className="flex justify-center my-3">
        <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-slate-100 dark:bg-slate-800/30 border border-slate-200 dark:border-slate-700">
          <Info className="w-3 h-3 text-muted-foreground/70" />
          <span className="text-[11px] text-muted-foreground/80">{msg.content}</span>
        </div>
      </div>
    );
  }

  return (
    <div className={`flex gap-2.5 mb-4 ${isUser ? 'flex-row-reverse' : 'flex-row'}`}>
      {!isUser && (
        <div className={`shrink-0 w-8 h-8 rounded-full flex items-center justify-center mt-5 ${
          msg.sender_type === 'ai'
            ? 'bg-violet-500/10 border border-violet-500/20'
            : 'bg-blue-500/10 border border-blue-500/20'
        }`}>
          {msg.sender_type === 'ai' ? (
            <Bot className="w-4 h-4 text-violet-400" />
          ) : (
            <Headset className="w-4 h-4 text-blue-400" />
          )}
        </div>
      )}
      <div className={`max-w-[78%] flex flex-col ${isUser ? 'items-end' : 'items-start'}`}>
        {!isUser && (
          <span className="text-[10px] font-semibold text-muted-foreground mb-1 px-1 uppercase tracking-wider">
            {msg.sender_type === 'ai' ? 'AI Assistant' : 'Support'}
          </span>
        )}
        <div
          className={`rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${
            isUser
              ? 'bg-gradient-to-br from-blue-500 to-blue-600 text-white rounded-br-md shadow-sm shadow-blue-500/20'
              : 'bg-slate-100 dark:bg-slate-800/50 text-foreground rounded-bl-md border border-slate-200 dark:border-white/8'
          }`}
        >
          <p className="whitespace-pre-wrap break-words">{msg.content}</p>
        </div>
        <span className={`text-[10px] text-muted-foreground/60 mt-1 px-1 ${isUser ? 'text-right' : 'text-left'}`}>
          {timestamp}
        </span>
      </div>
    </div>
  );
}
