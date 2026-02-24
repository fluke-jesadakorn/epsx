'use client';

import type { ChatAttachment, ChatMessage } from '@/shared/api/chat';
import { formatDistanceToNow } from 'date-fns';
import { Bot, Check, CheckCheck, Download, FileText, Headset, Info, X, ZoomIn, ZoomOut } from 'lucide-react';
import type React from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';

interface MsgItemProps {
  msg: ChatMessage;
  isUser: boolean;
  isRead?: boolean;
}

function getBackendUrl(): string {
  if (typeof window !== 'undefined') {
    try {
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const { getBackendUrl } = require('@/shared/utils/url-resolver') as { getBackendUrl: (ctx: string) => string };
      return getBackendUrl('client');
    } catch { /* ignore */ }
  }
  return '';
}

function SimpleMarkdown({ text }: { text: string }) {
  const lines = text.split('\n');
  return (
    <div className="whitespace-pre-wrap break-words">
      {lines.map((line, i) => {
        const parts: React.ReactNode[] = [];
        let rest = line;
        let key = 0;
        // inline: bold, italic, code, link
        const re = /(\*\*(.+?)\*\*|_(.+?)_|`([^`]+)`|\[([^\]]+)\]\(([^)]+)\))/g;
        let last = 0;
        let m: RegExpExecArray | null;
        while ((m = re.exec(rest)) !== null) {
          if (m.index > last) parts.push(rest.slice(last, m.index));
          if (m[2] !== undefined) parts.push(<strong key={key++} className="font-bold">{m[2]}</strong>);
          else if (m[3] !== undefined) parts.push(<em key={key++} className="italic">{m[3]}</em>);
          else if (m[4] !== undefined) parts.push(<code key={key++} className="bg-black/20 rounded px-1 text-xs font-mono">{m[4]}</code>);
          else if (m[5] !== undefined) {
            try {
              const url = m[6] as string;
              const parsedUrl = new URL(url);
              const isTrusted = parsedUrl.hostname.endsWith('epsx.io') || parsedUrl.hostname === 'localhost';
              if (isTrusted) {
                parts.push(<a key={key++} href={url} target="_blank" rel="noopener noreferrer" className="underline opacity-90">{m[5]}</a>);
              } else {
                parts.push(<span key={key++} className="text-red-400/80 italic text-xs" title="External links are restricted for security reasons">[External Link Removed]</span>);
              }
            } catch {
              parts.push(<span key={key++}>{m[5]}</span>);
            }
          }
          last = m.index + m[0].length;
        }
        if (last < rest.length) parts.push(rest.slice(last));
        return <p key={i} className="mb-1 last:mb-0">{parts.length > 0 ? parts : '\u00a0'}</p>;
      })}
    </div>
  );
}

function ImageLightbox({ src, alt, onClose }: { src: string; alt: string; onClose: () => void }) {
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const dragStart = useRef({ x: 0, y: 0 });

  const zoomIn = useCallback(() => setZoom(z => Math.min(z + 0.5, 4)), []);
  const zoomOut = useCallback(() => setZoom(z => {
    const next = Math.max(z - 0.5, 0.5);
    if (next <= 1) setPan({ x: 0, y: 0 });
    return next;
  }), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      if (e.key === '+' || e.key === '=') zoomIn();
      if (e.key === '-') zoomOut();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose, zoomIn, zoomOut]);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.stopPropagation();
    setZoom(z => {
      const next = Math.max(0.5, Math.min(4, z * (1 - e.deltaY * 0.001)));
      if (next <= 1) setPan({ x: 0, y: 0 });
      return next;
    });
  }, []);

  const handlePointerDown = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    if (zoom <= 1) return;
    setIsDragging(true);
    dragStart.current = { x: e.clientX - pan.x, y: e.clientY - pan.y };
    e.currentTarget.setPointerCapture(e.pointerId);
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    if (!isDragging) return;
    setPan({ x: e.clientX - dragStart.current.x, y: e.clientY - dragStart.current.y });
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    setIsDragging(false);
    e.currentTarget.releasePointerCapture(e.pointerId);
  };

  return (
    <div
      className="fixed inset-0 z-[9999] bg-black/85 backdrop-blur-sm flex items-center justify-center overflow-hidden"
      onClick={onClose}
      onWheel={handleWheel}
    >
      {/* Controls */}
      <div className="absolute top-4 right-4 flex items-center gap-1.5 z-10">
        <button
          onClick={e => { e.stopPropagation(); zoomIn(); }}
          className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors"
        >
          <ZoomIn className="w-4 h-4" />
        </button>
        <button
          onClick={e => { e.stopPropagation(); zoomOut(); }}
          className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors"
        >
          <ZoomOut className="w-4 h-4" />
        </button>
        <button
          onClick={e => { e.stopPropagation(); onClose(); }}
          className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors ml-1"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Zoom label */}
      <div className="absolute bottom-4 inset-x-0 text-center pointer-events-none z-10">
        <span className="text-white/40 text-xs px-3 py-1.5 rounded-full bg-black/50 backdrop-blur-md">
          {Math.round(zoom * 100)}% · scroll to zoom · drag to pan · esc to close
        </span>
      </div>

      {/* Image Container */}
      <div
        className="w-full h-full flex items-center justify-center"
      >
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={src}
          alt={alt}
          draggable={false}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerUp}
          onClick={e => e.stopPropagation()}
          style={{
            transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
            transition: isDragging ? 'none' : 'transform 0.1s ease-out',
            cursor: zoom > 1 ? (isDragging ? 'grabbing' : 'grab') : 'default',
            touchAction: 'none'
          }}
          className="max-w-[90vw] max-h-[90vh] object-contain rounded-xl select-none"
        />
      </div>
    </div>
  );
}

function AttachmentView({ att }: { att: ChatAttachment }) {
  const [open, setOpen] = useState(false);
  const isImage = att.file_type.startsWith('image/');
  const src = att.url.startsWith('/api') ? `${getBackendUrl()}${att.url}` : att.url;

  if (isImage) {
    return (
      <>
        <button
          type="button"
          onClick={() => setOpen(true)}
          className="block mt-2 cursor-zoom-in focus:outline-none"
        >
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src={src}
            alt={att.filename}
            className="max-w-full max-h-48 rounded-xl border border-white/10 object-cover hover:opacity-90 transition-opacity"
          />
        </button>
        {open && <ImageLightbox src={src} alt={att.filename} onClose={() => setOpen(false)} />}
      </>
    );
  }

  return (
    <a
      href={src}
      target="_blank"
      rel="noopener noreferrer"
      download={att.filename}
      className="flex items-center gap-2 mt-2 px-3 py-2 rounded-xl bg-black/10 hover:bg-black/20 transition-colors border border-white/10"
    >
      <FileText className="w-4 h-4 shrink-0" />
      <div className="min-w-0 flex-1">
        <p className="text-xs font-medium truncate">{att.filename}</p>
        <p className="text-[10px] opacity-60">{(att.size / 1024).toFixed(1)} KB</p>
      </div>
      <Download className="w-3.5 h-3.5 shrink-0 opacity-60" />
    </a>
  );
}

export function ChatMessageItem({ msg, isUser, isRead }: MsgItemProps) {
  const timestamp = formatDistanceToNow(new Date(msg.created_at), { addSuffix: true });
  const attachments = msg.metadata?.attachments ?? [];
  const isAttachmentOnly = msg.content.startsWith('[attachment:') && attachments.length > 0;

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
        <div className={`shrink-0 w-8 h-8 rounded-full flex items-center justify-center mt-5 ${msg.sender_type === 'ai'
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
          className={`rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${isUser
            ? 'bg-gradient-to-br from-blue-500 to-blue-600 text-white rounded-br-md shadow-sm shadow-blue-500/20'
            : 'bg-slate-100 dark:bg-slate-800/50 text-foreground rounded-bl-md border border-slate-200 dark:border-white/8'
            }`}
        >
          {!isAttachmentOnly && (
            <SimpleMarkdown text={msg.content} />
          )}
          {attachments.map((att, i) => (
            <AttachmentView key={i} att={att} />
          ))}
        </div>
        <div className={`flex items-center gap-1 mt-1 px-1 ${isUser ? 'flex-row-reverse' : 'flex-row'}`}>
          <span className="text-[10px] text-muted-foreground/60">{timestamp}</span>
          {isUser && (
            isRead
              ? <CheckCheck className="w-3.5 h-3.5 text-blue-400" />
              : <Check className="w-3.5 h-3.5 text-muted-foreground/40" />
          )}
        </div>
      </div>
    </div>
  );
}
