'use client';

import type { ChatAttachment, ChatMessage } from '@/shared/api/chat';
import { ChatMarkdown } from '@/shared/components/chat/chat-markdown';
import { ImageLightbox } from '@/shared/components/media/image-lightbox';
import { formatDistanceToNow } from 'date-fns';
import { Bot, Check, CheckCheck, Download, FileText, Headset, Info } from 'lucide-react';
import { useState } from 'react';

interface MsgItemProps {
  msg: ChatMessage;
  isUser: boolean;
  isRead?: boolean;
}

function toServeUrl(url: string): string {
  try {
    const parts = new URL(url).pathname.split('/').filter(Boolean);
    if (parts.length >= 3 && parts[0] === 'chat') {
      return `/api/chat-files/${parts[1]}/${parts[2]}`;
    }
  } catch { /* ignore */ }
  return url;
}

function AttachmentView({ att }: { att: ChatAttachment }) {
  const [open, setOpen] = useState(false);
  const isImage = att.file_type.startsWith('image/');
  const src = toServeUrl(att.url);
  const preview = toServeUrl(att.thumb_url ?? att.url);

  if (isImage) {
    return (
      <>
        <button
          type="button"
          onClick={() => setOpen(true)}
          className="block mt-2 cursor-zoom-in focus:outline-none rounded-xl overflow-hidden"
        >
          <img
            src={preview}
            alt={att.filename}
            loading="eager"
            className="max-w-full max-h-48 rounded-xl border border-white/15 object-cover hover:opacity-90 transition-opacity shadow-sm"
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
      className="flex items-center gap-2 mt-2 px-3 py-2 rounded-xl bg-black/10 hover:bg-black/18 transition-colors border border-white/12"
    >
      <FileText className="w-4 h-4 shrink-0" />
      <div className="min-w-0 flex-1">
        <p className="text-xs font-medium truncate">{att.filename}</p>
        <p className="text-[10px] opacity-55">{(att.size / 1024).toFixed(1)} KB</p>
      </div>
      <Download className="w-3.5 h-3.5 shrink-0 opacity-55" />
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
        <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-white/70 dark:bg-slate-800/40 border border-slate-200/70 dark:border-white/8 backdrop-blur-sm shadow-sm">
          <Info className="w-3 h-3 text-muted-foreground/60" />
          <span className="text-[11px] text-muted-foreground/70">{msg.content}</span>
        </div>
      </div>
    );
  }

  return (
    <div className={`flex gap-2.5 mb-3 ${isUser ? 'flex-row-reverse' : 'flex-row'}`}>
      {!isUser && (
        <div className={`shrink-0 w-8 h-8 rounded-full flex items-center justify-center mt-5 shadow-sm ${
          msg.sender_type === 'ai'
            ? 'bg-gradient-to-br from-violet-500/20 to-purple-500/20 border border-violet-500/25'
            : 'bg-gradient-to-br from-[#7645d9]/15 to-[#1fc7d4]/15 border border-[#7645d9]/25'
        }`}>
          {msg.sender_type === 'ai' ? (
            <Bot className="w-4 h-4 text-violet-400" />
          ) : (
            <Headset className="w-4 h-4 text-[#7645d9]" />
          )}
        </div>
      )}
      <div className={`max-w-[78%] flex flex-col ${isUser ? 'items-end' : 'items-start'}`}>
        {!isUser && (
          <span className="text-[10px] font-bold text-muted-foreground/50 mb-1 px-1 uppercase tracking-wider">
            {msg.sender_type === 'ai' ? 'AI Assistant' : 'Support'}
          </span>
        )}
        <div
          className={`rounded-2xl px-4 py-2.5 text-sm leading-relaxed shadow-sm ${isUser
            ? 'bg-gradient-to-br from-[#7645d9] to-[#5a33b8] text-white rounded-br-md shadow-[#7645d9]/20'
            : 'bg-white/90 dark:bg-slate-800/65 text-foreground rounded-bl-md border border-slate-200/70 dark:border-white/8 backdrop-blur-sm'
          }`}
        >
          {!isAttachmentOnly && (
            <ChatMarkdown text={msg.content} />
          )}
          {attachments.map((att, i) => (
            <AttachmentView key={i} att={att} />
          ))}
        </div>
        <div className={`flex items-center gap-1 mt-1 px-1 ${isUser ? 'flex-row-reverse' : 'flex-row'}`}>
          <span className="text-[10px] text-muted-foreground/40">{timestamp}</span>
          {isUser && (
            isRead
              ? <CheckCheck className="w-3.5 h-3.5 text-[#1fc7d4]" />
              : <Check className="w-3.5 h-3.5 text-muted-foreground/30" />
          )}
        </div>
      </div>
    </div>
  );
}
