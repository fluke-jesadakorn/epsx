'use client';

import { SlidersHorizontal } from 'lucide-react';
import type { ChatTopic } from '@/shared/api/chat';

interface Props {
  status: string;
  topicId: string;
  topics: ChatTopic[];
  onStatusChange: (s: string) => void;
  onTopicChange: (t: string) => void;
}

export function ChatFilterBar({ status, topicId, topics, onStatusChange, onTopicChange }: Props) {
  return (
    <div className="flex items-center gap-2 mb-3 p-2.5 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-700 rounded-xl backdrop-blur-sm">
      <SlidersHorizontal className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
      <select
        value={status}
        onChange={e => onStatusChange(e.target.value)}
        className="flex-1 px-2.5 py-1.5 text-xs font-medium bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700 rounded-lg text-foreground focus:outline-none focus:border-violet-500/50 focus:ring-1 focus:ring-violet-500/20 transition-all cursor-pointer"
      >
        <option value="">All Status</option>
        <option value="open">Open</option>
        <option value="in_progress">In Progress</option>
        <option value="resolved">Resolved</option>
        <option value="closed">Closed</option>
      </select>
      <select
        value={topicId}
        onChange={e => onTopicChange(e.target.value)}
        className="flex-1 px-2.5 py-1.5 text-xs font-medium bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700 rounded-lg text-foreground focus:outline-none focus:border-violet-500/50 focus:ring-1 focus:ring-violet-500/20 transition-all cursor-pointer"
      >
        <option value="">All Topics</option>
        {topics.map(t => (
          <option key={t.id} value={t.id}>
            {t.label}
          </option>
        ))}
      </select>
    </div>
  );
}
