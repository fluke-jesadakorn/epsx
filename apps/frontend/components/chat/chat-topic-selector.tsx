'use client';

import type { ChatTopic } from '@/shared/api/chat';
import { TurnstileWidget } from '@/shared/components/turnstile-widget';
import {
  ArrowLeft,
  BarChart3,
  Bug,
  ChevronRight,
  CreditCard,
  HelpCircle,
  Lightbulb,
  MessageCircle,
  Paperclip,
  Send,
  Settings,
  Shield,
  User,
  X,
  Zap,
  type LucideIcon
} from 'lucide-react';
import { useCallback, useRef, useState, type DragEvent } from 'react';

const ICON_MAP: Record<string, LucideIcon> = {
  'message-circle': MessageCircle,
  'credit-card': CreditCard,
  user: User,
  'bar-chart': BarChart3,
  bug: Bug,
  lightbulb: Lightbulb,
  'help-circle': HelpCircle,
  settings: Settings,
  shield: Shield,
  zap: Zap,
};

const TOPIC_COLORS: Record<string, { bg: string; icon: string; border: string; glow: string }> = {
  'message-circle': { bg: 'bg-blue-500/10', icon: 'text-blue-400', border: 'border-blue-500/15 hover:border-blue-500/40', glow: 'group-hover:shadow-blue-500/5' },
  'credit-card': { bg: 'bg-emerald-500/10', icon: 'text-emerald-400', border: 'border-emerald-500/15 hover:border-emerald-500/40', glow: 'group-hover:shadow-emerald-500/5' },
  user: { bg: 'bg-violet-500/10', icon: 'text-violet-400', border: 'border-violet-500/15 hover:border-violet-500/40', glow: 'group-hover:shadow-violet-500/5' },
  'bar-chart': { bg: 'bg-amber-500/10', icon: 'text-amber-400', border: 'border-amber-500/15 hover:border-amber-500/40', glow: 'group-hover:shadow-amber-500/5' },
  bug: { bg: 'bg-red-500/10', icon: 'text-red-400', border: 'border-red-500/15 hover:border-red-500/40', glow: 'group-hover:shadow-red-500/5' },
  lightbulb: { bg: 'bg-yellow-500/10', icon: 'text-yellow-400', border: 'border-yellow-500/15 hover:border-yellow-500/40', glow: 'group-hover:shadow-yellow-500/5' },
};

const DEFAULT_COLOR = { bg: 'bg-muted', icon: 'text-muted-foreground', border: 'border-border hover:border-primary/40', glow: '' };

interface ConversationCreateOptions {
  topicId: string;
  subject: string;
  message: string;
  turnstileToken?: string;
  file?: File;
}

interface TopicSelectorProps {
  topics: ChatTopic[];
  onSelect: (opts: ConversationCreateOptions) => void;
  compact?: boolean;
}

export function ChatTopicSelector({ topics, onSelect, compact }: TopicSelectorProps) {
  const [selected, setSelected] = useState<string | null>(null);
  const [subject, setSubject] = useState('');
  const [message, setMessage] = useState('');
  const [sending, setSending] = useState(false);
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);
  const [pendingFile, setPendingFile] = useState<File | null>(null);
  const [dragging, setDragging] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);

  const handleSubmit = useCallback(() => {
    if (selected && subject.trim() && message.trim() && !sending && turnstileToken !== null) {
      setSending(true);
      onSelect({ topicId: selected, subject: subject.trim(), message: message.trim(), turnstileToken, file: pendingFile ?? undefined });
    }
  }, [selected, subject, message, onSelect, sending, turnstileToken, pendingFile]);

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) { setPendingFile(file); }
    e.target.value = '';
  }, []);

  const handleDrop = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setDragging(false);
    const file = e.dataTransfer.files[0];
    if (file) { setPendingFile(file); }
  }, []);

  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setDragging(true);
  }, []);

  const handleDragLeave = useCallback(() => setDragging(false), []);

  const getIcon = (iconName: string | null) => {
    if (!iconName) { return HelpCircle; }
    return ICON_MAP[iconName] ?? HelpCircle;
  };

  const getColor = (iconName: string | null) => {
    if (!iconName) { return DEFAULT_COLOR; }
    return TOPIC_COLORS[iconName] ?? DEFAULT_COLOR;
  };

  if (selected) {
    const topic = topics.find((t) => t.id === selected);
    const Icon = getIcon(topic?.icon ?? null);
    const color = getColor(topic?.icon ?? null);

    return (
      <div className="flex-1 flex flex-col p-4 overflow-hidden">
        <button
          onClick={() => { setSelected(null); setSending(false); }}
          className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground mb-3 w-fit transition-colors"
        >
          <ArrowLeft className="w-3.5 h-3.5" />
          Back to topics
        </button>

        <div className="flex items-center gap-3 mb-3 p-2.5 rounded-xl bg-slate-50 dark:bg-slate-800/30 border border-slate-200 dark:border-white/8">
          <div className={`w-9 h-9 rounded-lg ${color.bg} flex items-center justify-center shrink-0`}>
            <Icon className={`w-4 h-4 ${color.icon}`} />
          </div>
          <div>
            <h3 className="font-semibold text-sm">{topic?.label}</h3>
            {topic?.description && (
              <p className="text-[11px] text-muted-foreground mt-0.5">{topic.description}</p>
            )}
          </div>
        </div>

        <div className="flex flex-col flex-1 min-h-0 gap-2.5">
          <div>
            <label className="text-[11px] font-semibold text-muted-foreground mb-1 block uppercase tracking-wider">Subject</label>
            <input
              type="text"
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
              placeholder="Brief summary of your issue"
              className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-white/8 bg-white dark:bg-slate-800/30 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500/30 transition-all placeholder:text-muted-foreground/40"
              autoFocus
            />
          </div>
          <div className="flex-1 flex flex-col min-h-0">
            <label className="text-[11px] font-semibold text-muted-foreground mb-1 block uppercase tracking-wider">Message</label>
            <textarea
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Describe your issue in detail..."
              className="flex-1 w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-white/8 bg-white dark:bg-slate-800/30 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500/30 transition-all resize-none placeholder:text-muted-foreground/40 min-h-[120px]"
            />
          </div>

          <input
            ref={fileRef}
            type="file"
            accept="image/jpeg,image/png,image/gif,image/webp,application/pdf"
            onChange={handleFileChange}
            className="hidden"
          />
          {pendingFile !== null ? (
            <div className="flex items-center gap-2 px-3 py-2 bg-blue-500/10 border border-blue-500/20 rounded-xl text-xs text-blue-400">
              <Paperclip className="w-3.5 h-3.5 shrink-0" />
              <span className="flex-1 truncate font-medium">{pendingFile.name}</span>
              <button onClick={() => setPendingFile(null)} className="shrink-0 hover:opacity-70">
                <X className="w-3.5 h-3.5" />
              </button>
            </div>
          ) : (
            <div
              onClick={() => { if (!sending) { fileRef.current?.click(); } }}
              onDrop={handleDrop}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              className={`flex flex-col items-center justify-center gap-1.5 py-3 rounded-xl border-2 border-dashed cursor-pointer transition-all ${sending ? 'opacity-40 cursor-not-allowed' : ''} ${dragging ? 'border-blue-500/60 bg-blue-500/10' : 'border-slate-200 dark:border-white/10 hover:border-blue-500/40 hover:bg-slate-50 dark:hover:bg-slate-800/30'}`}
            >
              <Paperclip className={`w-4 h-4 ${dragging ? 'text-blue-400' : 'text-muted-foreground/40'}`} />
              <p className="text-[11px] font-medium text-muted-foreground/60">
                {dragging ? 'Drop file here' : 'Attach a screenshot or file'}
              </p>
              <p className="text-[10px] text-muted-foreground/30">JPG, PNG, GIF, WebP, PDF · Max 5MB</p>
            </div>
          )}
        </div>

        <TurnstileWidget
          action="chat"
          onSuccess={setTurnstileToken}
          onExpire={() => setTurnstileToken(null)}
          className="mt-2"
        />

        <button
          onClick={handleSubmit}
          disabled={!subject.trim() || !message.trim() || sending || turnstileToken === null}
          className="w-full mt-2 py-2.5 rounded-xl bg-gradient-to-r from-blue-500 to-blue-600 text-white font-semibold text-sm hover:from-blue-400 hover:to-blue-500 disabled:opacity-40 disabled:cursor-not-allowed disabled:from-muted disabled:to-muted disabled:text-muted-foreground transition-all shadow-sm shadow-blue-500/20 flex items-center justify-center gap-2"
        >
          <Send className="w-4 h-4" />
          {sending ? 'Starting...' : 'Start Conversation'}
        </button>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-4">
      <h3 className="font-bold text-sm mb-0.5">How can we help?</h3>
      <p className="text-[11px] text-muted-foreground mb-3">Select a topic to get started</p>
      <div className="grid gap-1.5">
        {topics.map((topic) => {
          const Icon = getIcon(topic.icon);
          const color = getColor(topic.icon);
          return (
            <button
              key={topic.id}
              onClick={() => setSelected(topic.id)}
              className={`group p-3 rounded-xl border ${color.border} bg-white dark:bg-slate-800/40 hover:bg-slate-50 dark:hover:bg-slate-800/60 transition-all text-left`}
            >
              <div className="flex items-center gap-2.5">
                <div className={`w-8 h-8 rounded-lg ${color.bg} flex items-center justify-center shrink-0 transition-transform group-hover:scale-110`}>
                  <Icon className={`w-4 h-4 ${color.icon}`} />
                </div>
                <div className="flex-1 min-w-0">
                  <h4 className="font-semibold text-[13px]">{topic.label}</h4>
                  {topic.description && (
                    <p className="text-[10px] text-muted-foreground mt-0.5 line-clamp-1">{topic.description}</p>
                  )}
                </div>
                <ChevronRight className="w-3.5 h-3.5 text-muted-foreground/30 group-hover:text-muted-foreground/60 transition-colors shrink-0" />
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}

