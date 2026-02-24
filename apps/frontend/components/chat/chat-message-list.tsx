'use client';

import { useEffect, useRef } from 'react';
import type { ChatMessage } from '@/shared/api/chat';
import { ChatMessageItem } from './chat-message-item';
import { format, isSameDay } from 'date-fns';
import { MessageCircle } from 'lucide-react';

interface MsgListProps {
  msgs: ChatMessage[];
  userAddr?: string;
  agentTyping?: boolean;
  readUpToId?: string | null;
}

export function ChatMessageList({ msgs, userAddr, agentTyping, readUpToId }: MsgListProps) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [msgs, agentTyping]);

  if (msgs.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-center p-8">
        <div className="w-14 h-14 rounded-2xl bg-blue-500/10 flex items-center justify-center mb-4">
          <MessageCircle className="w-7 h-7 text-blue-400" />
        </div>
        <p className="text-sm font-medium text-foreground mb-1">No messages yet</p>
        <p className="text-xs text-muted-foreground/60">Type below to start the conversation</p>
      </div>
    );
  }

  let lastDate: Date | null = null;
  let readReached = false;

  return (
    <div className="flex-1 overflow-y-auto px-4 py-4 bg-slate-50/50 dark:bg-slate-950/20">
      {msgs.map((msg) => {
        const msgDate = new Date(msg.created_at);
        const showDateSep = !lastDate || !isSameDay(lastDate, msgDate);
        lastDate = msgDate;
        const isUser = msg.sender_type === 'user' && msg.sender_address === userAddr;

        // Track whether agent has read past this message
        if (!readReached && readUpToId && msg.id === readUpToId) {
          readReached = true;
        }
        const isRead = isUser && (readReached || msg.id === readUpToId || msg.is_read);

        return (
          <div key={msg.id}>
            {showDateSep && (
              <div className="flex items-center gap-3 my-5">
                <div className="flex-1 h-px bg-border/40" />
                <span className="text-[10px] font-medium text-muted-foreground/60 uppercase tracking-widest">
                  {format(msgDate, 'MMM d, yyyy')}
                </span>
                <div className="flex-1 h-px bg-border/40" />
              </div>
            )}
            <ChatMessageItem msg={msg} isUser={isUser} isRead={isRead} />
          </div>
        );
      })}

      {agentTyping === true && (
        <div className="flex gap-2.5 mb-4">
          <div className="shrink-0 w-8 h-8 rounded-full bg-blue-500/10 border border-blue-500/20 flex items-center justify-center">
            <MessageCircle className="w-4 h-4 text-blue-400" />
          </div>
          <div className="bg-slate-100 dark:bg-slate-800/50 border border-slate-200 dark:border-white/8 rounded-2xl rounded-bl-md px-4 py-3 flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/50 animate-bounce" style={{ animationDelay: '0ms' }} />
            <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/50 animate-bounce" style={{ animationDelay: '150ms' }} />
            <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/50 animate-bounce" style={{ animationDelay: '300ms' }} />
          </div>
        </div>
      )}

      <div ref={endRef} />
    </div>
  );
}
