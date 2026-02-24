'use client';

import { TurnstileWidget } from '@/shared/components/turnstile-widget';
import { Paperclip, Send, X } from 'lucide-react';
import { useCallback, useEffect, useRef, useState, type KeyboardEvent } from 'react';

interface InputProps {
  onSend: (content: string, turnstileToken: string) => void;
  onUpload?: (file: File) => void;
  onTyping?: (isTyping: boolean) => void;
  disabled?: boolean;
  placeholder?: string;
}

export function ChatInput({ onSend, onUpload, onTyping, disabled = false, placeholder = 'Type a message...' }: InputProps) {
  const [val, setVal] = useState('');
  const [pendingFile, setPendingFile] = useState<File | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const typingTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);
  const [turnstileKey, setTurnstileKey] = useState(0);

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

  const emitTyping = useCallback((typing: boolean) => {
    if (isTypingRef.current === typing) return;
    isTypingRef.current = typing;
    onTyping?.(typing);
  }, [onTyping]);

  const handleChange = useCallback((v: string) => {
    setVal(v);
    if (v.length > 0) {
      emitTyping(true);
      if (typingTimerRef.current) clearTimeout(typingTimerRef.current);
      typingTimerRef.current = setTimeout(() => emitTyping(false), 2000);
    } else {
      emitTyping(false);
    }
  }, [emitTyping]);

  const handleSend = useCallback(() => {
    const trimmed = val.trim();
    if (pendingFile && onUpload) {
      onUpload(pendingFile);
      setPendingFile(null);
      setVal('');
      emitTyping(false);
      return;
    }
    if (trimmed && !disabled && turnstileToken !== null) {
      onSend(trimmed, turnstileToken);
      setVal('');
      setTurnstileToken(null);
      setTurnstileKey(k => k + 1);
      emitTyping(false);
    }
  }, [val, disabled, onSend, pendingFile, onUpload, emitTyping, turnstileToken]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setPendingFile(file);
    }
    // Reset input so same file can be picked again
    e.target.value = '';
  }, []);

  const canSend = (val.trim().length > 0 || pendingFile !== null) && !disabled && (pendingFile !== null || turnstileToken !== null);

  return (
    <div className="p-3 border-t border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-950/40">
      {pendingFile !== null && (
        <div className="flex items-center gap-2 mb-2 px-3 py-2 bg-blue-500/10 border border-blue-500/20 rounded-xl text-xs text-blue-400">
          <Paperclip className="w-3.5 h-3.5 shrink-0" />
          <span className="flex-1 truncate">{pendingFile.name}</span>
          <button onClick={() => setPendingFile(null)} className="shrink-0 hover:opacity-70">
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}
      <div className="flex items-end gap-2 bg-slate-100 dark:bg-slate-800/30 rounded-2xl border border-slate-200 dark:border-white/8 px-3 py-2 focus-within:border-blue-500/30 focus-within:ring-2 focus-within:ring-blue-500/10 transition-all">
        {onUpload !== undefined && (
          <>
            <input
              ref={fileRef}
              type="file"
              accept="image/jpeg,image/png,image/gif,image/webp,application/pdf"
              onChange={handleFileChange}
              className="hidden"
            />
            <button
              onClick={() => fileRef.current?.click()}
              disabled={disabled}
              className="shrink-0 w-7 h-7 rounded-lg flex items-center justify-center text-muted-foreground/40 hover:text-muted-foreground hover:bg-slate-200 dark:hover:bg-slate-700 transition-all disabled:opacity-30"
              aria-label="Attach file"
            >
              <Paperclip className="w-4 h-4" />
            </button>
          </>
        )}
        <textarea
          ref={textareaRef}
          value={val}
          onChange={(e) => handleChange(e.target.value)}
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
            className={`w-8 h-8 rounded-xl flex items-center justify-center transition-all ${canSend
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
          Press Enter to send · Shift+Enter for new line · Supports **markdown**
        </p>
      )}

      {/* Turnstile Widget */}
      <TurnstileWidget
        key={turnstileKey}
        action="chat_message"
        onSuccess={setTurnstileToken}
        onExpire={() => setTurnstileToken(null)}
        className="mt-3 flex justify-center"
      />
    </div>
  );
}
