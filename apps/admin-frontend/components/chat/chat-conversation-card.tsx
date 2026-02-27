'use client';

import { Clock, Wallet } from 'lucide-react';
import type { ChatConversation, ChatTopic } from '@/shared/api/chat';
import { ChatStatusBadge } from './chat-status-badge';

interface Props {
  conv: ChatConversation;
  topics: ChatTopic[];
  selected: boolean;
  onClick: () => void;
}

function truncAddr(addr: string): string {
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
}

function timeAgo(date: string): string {
  const now = new Date();
  const past = new Date(date);
  const diff = now.getTime() - past.getTime();
  const mins = Math.floor(diff / 60000);
  const hrs = Math.floor(mins / 60);
  const days = Math.floor(hrs / 24);

  if (days > 0) {
    return `${days}d ago`;
  }
  if (hrs > 0) {
    return `${hrs}h ago`;
  }
  if (mins > 0) {
    return `${mins}m ago`;
  }
  return 'Just now';
}

export function ChatConversationCard({ conv, topics, selected, onClick }: Props) {
  const topic = topics.find(t => t.id === conv.topic_id);
  const unread = conv.unread_agent > 0;

  return (
    <button
      onClick={onClick}
      className={`w-full text-left p-3.5 rounded-xl transition-all ${
        selected
          ? 'bg-violet-500/10 border border-violet-500/25 shadow-sm shadow-violet-500/5'
          : 'bg-card border border-border/20 hover:bg-muted/30 hover:border-border/20'
      }`}
    >
      {/* Subject + unread */}
      <div className="flex items-start justify-between gap-2 mb-2">
        <p className={`text-sm leading-snug line-clamp-1 ${unread ? 'font-bold text-foreground' : 'font-semibold text-foreground/80'}`}>
          {conv.subject}
        </p>
        {unread && (
          <div className="flex-shrink-0 min-w-[22px] h-[22px] px-1 rounded-full bg-gradient-to-r from-violet-500 to-purple-500 text-white text-[10px] font-bold flex items-center justify-center shadow-sm shadow-violet-500/30">
            {conv.unread_agent > 9 ? '9+' : conv.unread_agent}
          </div>
        )}
      </div>

      {/* Wallet + Topic */}
      <div className="flex items-center gap-2 mb-2">
        <div className="flex items-center gap-1 text-muted-foreground">
          <Wallet className="w-3 h-3" />
          <span className="text-[11px] font-mono">{truncAddr(conv.wallet_address)}</span>
        </div>
        {topic && (
          <>
            <span className="text-muted-foreground/30 text-[10px]">|</span>
            <span className="text-[11px] font-semibold text-violet-400">{topic.label}</span>
          </>
        )}
      </div>

      {/* Status + time */}
      <div className="flex items-center justify-between">
        <ChatStatusBadge status={conv.status} />
        <div className="flex items-center gap-1 text-[10px] text-muted-foreground/60">
          <Clock className="w-2.5 h-2.5" />
          {timeAgo(conv.last_message_at)}
        </div>
      </div>
    </button>
  );
}
