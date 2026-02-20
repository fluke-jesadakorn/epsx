'use client';

import { useState, useCallback, useRef, useEffect, type KeyboardEvent } from 'react';
import { Send } from 'lucide-react';

interface InputProps {
  onSend: (content: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

export function ChatInput({ onSend, disabled = false, placeholder = 'Type a message...' }: InputProps) {
  const [val, setVal] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const autoResize = useCallback(() => {
    const el = textareaRef.current;
    if (el) {
      el.style.height = 'auto';
      el.style.height = `${Math.min(el.scrollHeight, 120)}px`;
    }
  }, []);

  useEffect(() => {
    autoResize();
  }, [val, autoResize]);

  const handleSend = useCallback(() => {
    const trimmed = val.trim();
    if (trimmed && !disabled) {
      onSend(trimmed);
      setVal('');
    }
  }, [val, disabled, onSend]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const canSend = val.trim().length > 0 && !disabled;

  return (
    <div className="p-3 border-t border-slate-200 dark:border-white/5 bg-slate-50 dark:bg-slate-950/40">
      <div className="flex items-end gap-2 bg-slate-100 dark:bg-slate-800/30 rounded-2xl border border-slate-200 dark:border-white/8 px-3 py-2 focus-within:border-blue-500/30 focus-within:ring-2 focus-within:ring-blue-500/10 transition-all">
        <textarea
          ref={textareaRef}
          value={val}
          onChange={(e) => setVal(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled}
          rows={1}
          className="flex-1 resize-none bg-transparent text-sm focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed placeholder:text-muted-foreground/40 py-1"
          style={{ minHeight: '28px' }}
        />
        <div className="flex items-center gap-1 shrink-0">
          <button
            onClick={handleSend}
            disabled={!canSend}
            className={`w-8 h-8 rounded-xl flex items-center justify-center transition-all ${
              canSend
                ? 'bg-blue-600 text-white hover:bg-blue-500 shadow-sm shadow-blue-500/20'
                : 'text-muted-foreground/30 cursor-not-allowed'
            }`}
            aria-label="Send message"
          >
            <Send className="w-4 h-4" />
          </button>
        </div>
      </div>
      {!disabled && (
        <p className="text-[10px] text-muted-foreground/40 mt-1.5 text-center">
          Press Enter to send, Shift+Enter for new line
        </p>
      )}
    </div>
  );
}
