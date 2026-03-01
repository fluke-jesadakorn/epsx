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

function AttachmentView({ att }: { att: ChatAttachment }) {
  const [open, setOpen] = useState(false);
  const isImage = att.file_type.startsWith('image/');
  const src = att.url;
  const preview = att.thumb_url ?? src;

  if (isImage) {
    return (
      <>
        <button
          type="button"
          onClick={() => setOpen(true)}
          className="block mt-2 cursor-zoom-in focus:outline-none"
        >
          <img
            src={preview}
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
            <ChatMarkdown text={msg.content} />
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
