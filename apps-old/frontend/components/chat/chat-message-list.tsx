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
        <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-[#7645d9]/10 to-[#1fc7d4]/10 border border-[#7645d9]/15 flex items-center justify-center mb-4 shadow-sm">
          <MessageCircle className="w-7 h-7 text-[#7645d9]/35" />
        </div>
        <p className="text-sm font-semibold text-foreground/55 mb-1.5">No messages yet</p>
        <p className="text-xs text-muted-foreground/35 max-w-[180px] leading-relaxed">Type your message below to start the conversation</p>
      </div>
    );
  }

  let lastDate: Date | null = null;
  let readReached = false;

  return (
    <div className="flex-1 overflow-y-auto scrollbar-thin flex flex-col">
      <div className="flex-1" />
      <div
        className="flex flex-col px-4 md:px-6 py-5"
        style={{
          backgroundImage: 'radial-gradient(circle, rgba(118,69,217,0.045) 1px, transparent 1px)',
          backgroundSize: '22px 22px',
        }}
      >
        {msgs.map((msg) => {
          const msgDate = new Date(msg.created_at);
          const showDateSep = !lastDate || !isSameDay(lastDate, msgDate);
          lastDate = msgDate;
          const isUser = msg.sender_type === 'user' && msg.sender_address === userAddr;

          if (!readReached && readUpToId !== null && readUpToId !== undefined && msg.id === readUpToId) {
            readReached = true;
          }
          const isRead = isUser && (readReached || msg.id === readUpToId || msg.is_read);

          return (
            <div key={msg.id}>
              {showDateSep && (
                <div className="flex items-center gap-3 my-6">
                  <div className="flex-1 h-px bg-gradient-to-r from-transparent via-slate-200 dark:via-white/8 to-transparent" />
                  <span className="text-[10px] font-semibold text-muted-foreground/45 uppercase tracking-[0.12em] px-2.5 py-1 rounded-full bg-white/70 dark:bg-white/5 border border-slate-200/60 dark:border-white/8 backdrop-blur-sm shadow-sm">
                    {format(msgDate, 'MMM d, yyyy')}
                  </span>
                  <div className="flex-1 h-px bg-gradient-to-r from-transparent via-slate-200 dark:via-white/8 to-transparent" />
                </div>
              )}
              <ChatMessageItem msg={msg} isUser={isUser} isRead={isRead} />
            </div>
          );
        })}

        {agentTyping === true && (
          <div className="flex gap-2.5 mb-3">
            <div className="shrink-0 w-8 h-8 rounded-full bg-gradient-to-br from-[#7645d9]/15 to-[#1fc7d4]/15 border border-[#7645d9]/20 flex items-center justify-center mt-4">
              <MessageCircle className="w-3.5 h-3.5 text-[#7645d9]" />
            </div>
            <div className="bg-white/90 dark:bg-slate-800/70 border border-slate-200/70 dark:border-white/8 rounded-2xl rounded-bl-md px-4 py-3 flex items-center gap-1.5 shadow-sm backdrop-blur-sm">
              <span className="w-1.5 h-1.5 rounded-full bg-[#7645d9]/50 animate-bounce" style={{ animationDelay: '0ms' }} />
              <span className="w-1.5 h-1.5 rounded-full bg-[#7645d9]/50 animate-bounce" style={{ animationDelay: '150ms' }} />
              <span className="w-1.5 h-1.5 rounded-full bg-[#7645d9]/50 animate-bounce" style={{ animationDelay: '300ms' }} />
            </div>
          </div>
        )}

        <div ref={endRef} />
      </div>
    </div>
  );
}
