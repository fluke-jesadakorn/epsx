'use client';

import { Paperclip, Send, X } from 'lucide-react';
import { useCallback, useEffect, useRef, useState, type KeyboardEvent } from 'react';

interface InputProps {
  onSend: (content: string, turnstileToken: string) => void;
  onUpload?: (file: File) => void;
  onTyping?: (isTyping: boolean) => void;
  disabled?: boolean;
  placeholder?: string;
  turnstileToken: string | null;
  onTokenUsed: () => void;
}

export function ChatInput({ onSend, onUpload, onTyping, disabled = false, placeholder = 'Type a message...', turnstileToken, onTokenUsed }: InputProps) {
  const [val, setVal] = useState('');
  const [pendingFile, setPendingFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const typingTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);

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
    if (isTypingRef.current === typing) { return; }
    isTypingRef.current = typing;
    onTyping?.(typing);
  }, [onTyping]);

  const handleChange = useCallback((v: string) => {
    setVal(v);
    if (v.length > 0) {
      emitTyping(true);
      if (typingTimerRef.current) { clearTimeout(typingTimerRef.current); }
      typingTimerRef.current = setTimeout(() => emitTyping(false), 2000);
    } else {
      emitTyping(false);
    }
  }, [emitTyping]);

  const clearPending = useCallback(() => {
    setPendingFile(null);
    if (previewUrl !== null) { URL.revokeObjectURL(previewUrl); setPreviewUrl(null); }
  }, [previewUrl]);

  const handleSend = useCallback(() => {
    const trimmed = val.trim();
    if (pendingFile !== null && onUpload !== undefined) {
      onUpload(pendingFile);
      clearPending();
      setVal('');
      emitTyping(false);
      return;
    }
    if (trimmed && !disabled && turnstileToken !== null) {
      onSend(trimmed, turnstileToken);
      setVal('');
      onTokenUsed();
      emitTyping(false);
    }
  }, [val, disabled, onSend, pendingFile, onUpload, emitTyping, turnstileToken, onTokenUsed, clearPending]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  useEffect(() => {
    return () => { if (previewUrl !== null) { URL.revokeObjectURL(previewUrl); } };
  }, [previewUrl]);

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setPendingFile(file);
      setPreviewUrl(file.type.startsWith('image/') ? URL.createObjectURL(file) : null);
    }
    e.target.value = '';
  }, []);

  const canSend = (val.trim().length > 0 || pendingFile !== null) && !disabled && (pendingFile !== null || turnstileToken !== null);

  return (
    <div className="flex-shrink-0 px-4 md:px-5 py-3 bg-white/90 dark:bg-slate-900/90 backdrop-blur-xl border-t border-slate-200/60 dark:border-white/5">
      {pendingFile !== null && (
        <div className="mb-2">
          {previewUrl !== null ? (
            <div className="relative inline-block">
              <img src={previewUrl} alt={pendingFile.name} className="max-h-28 max-w-full rounded-xl border border-[#7645d9]/20 object-cover shadow-sm" />
              <button
                onClick={clearPending}
                className="absolute -top-1.5 -right-1.5 w-5 h-5 rounded-full bg-slate-900 dark:bg-slate-700 border border-slate-600 flex items-center justify-center hover:bg-red-500 transition-colors shadow-sm"
              >
                <X className="w-2.5 h-2.5 text-white" />
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-2 px-3 py-2 bg-[#7645d9]/8 border border-[#7645d9]/20 rounded-xl text-xs text-[#7645d9] dark:text-violet-400">
              <Paperclip className="w-3.5 h-3.5 shrink-0" />
              <span className="flex-1 truncate font-medium">{pendingFile.name}</span>
              <button onClick={clearPending} className="shrink-0 hover:opacity-60 transition-opacity"><X className="w-3.5 h-3.5" /></button>
            </div>
          )}
        </div>
      )}

      <div className={`flex items-end gap-2.5 bg-slate-100/80 dark:bg-white/[0.06] rounded-2xl border px-3.5 py-2 transition-all ${
        disabled
          ? 'border-slate-200/60 dark:border-white/5 opacity-60'
          : 'border-slate-200/80 dark:border-white/10 focus-within:border-[#7645d9]/40 focus-within:ring-2 focus-within:ring-[#7645d9]/10 focus-within:bg-white/90 dark:focus-within:bg-white/[0.08]'
      }`}>
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
              className="shrink-0 w-7 h-7 rounded-lg flex items-center justify-center text-muted-foreground/35 hover:text-muted-foreground hover:bg-slate-200/70 dark:hover:bg-white/10 transition-all disabled:opacity-25"
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
          className="flex-1 resize-none bg-transparent text-sm focus:outline-none disabled:cursor-not-allowed placeholder:text-muted-foreground/35 py-1 leading-relaxed"
          style={{ minHeight: '28px' }}
        />
        <button
          onClick={handleSend}
          disabled={!canSend}
          className={`shrink-0 w-8 h-8 rounded-xl flex items-center justify-center transition-all ${
            canSend
              ? 'bg-gradient-to-br from-[#7645d9] to-[#5a33b8] text-white hover:from-[#8755e8] hover:to-[#6840cc] shadow-md shadow-[#7645d9]/25 active:scale-95'
              : 'text-muted-foreground/25 cursor-not-allowed'
          }`}
          aria-label="Send message"
        >
          <Send className="w-4 h-4" />
        </button>
      </div>
      {!disabled && (
        <p className="text-[10px] text-muted-foreground/30 mt-1.5 text-center">
          Enter to send · Shift+Enter for new line · **markdown** supported
        </p>
      )}
    </div>
  );
}
