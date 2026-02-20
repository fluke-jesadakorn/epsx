'use client';

import { MessageCircle, X } from 'lucide-react';

interface BubbleProps {
  unread: number;
  onClick: () => void;
  isOpen: boolean;
}

export function ChatBubble({ unread, onClick, isOpen }: BubbleProps) {
  const hasUnread = unread > 0;

  return (
    <button
      onClick={onClick}
      className={`group relative w-14 h-14 rounded-full shadow-lg transition-all duration-300 flex items-center justify-center ${
        isOpen
          ? 'bg-card/90 backdrop-blur-md border border-border hover:bg-muted scale-95'
          : 'bg-gradient-to-br from-blue-500 via-blue-600 to-indigo-600 hover:from-blue-400 hover:via-blue-500 hover:to-indigo-500 hover:shadow-blue-500/25 hover:shadow-xl hover:scale-105'
      }`}
      aria-label={isOpen ? 'Close support chat' : 'Open support chat'}
    >
      {/* Subtle glow ring for unread */}
      {!isOpen && hasUnread && (
        <span className="absolute inset-0 rounded-full border-2 border-blue-400/30 animate-ping" />
      )}

      {isOpen ? (
        <X className="w-5 h-5 text-foreground transition-transform duration-200 group-hover:rotate-90" />
      ) : (
        <MessageCircle className="w-6 h-6 text-white transition-transform duration-200 group-hover:scale-110" />
      )}

      {hasUnread && !isOpen && (
        <span className="absolute -top-1.5 -right-1.5 min-w-[22px] h-[22px] px-1.5 bg-red-500 rounded-full text-white text-[11px] flex items-center justify-center font-bold border-2 border-background shadow-sm">
          {unread > 9 ? '9+' : unread}
        </span>
      )}
    </button>
  );
}
