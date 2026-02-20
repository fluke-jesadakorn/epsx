'use client';

import { useState, useEffect, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import type { ChatConversation, ChatTopic } from '@/shared/api/chat';
import { getTopicsAction, listConversationsAction } from '@/app/actions/chat';
import { ChatStatusBadge } from '@/components/chat/chat-status-badge';
import { formatDistanceToNow } from 'date-fns';
import { useSharedAuth } from '@/shared/components/auth';
import { ArrowLeft, Loader2, Inbox, ChevronRight, SlidersHorizontal } from 'lucide-react';
import Link from 'next/link';

export default function ChatHistoryPage() {
  const router = useRouter();
  const { isAuthenticated } = useSharedAuth();
  const [convos, setConvos] = useState<ChatConversation[]>([]);
  const [topics, setTopics] = useState<ChatTopic[]>([]);
  const [loading, setLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [topicFilter, setTopicFilter] = useState<string>('all');

  useEffect(() => {
    const load = async () => {
      const [convosData, topicsData] = await Promise.all([
        listConversationsAction(),
        getTopicsAction()
      ]);
      setConvos(convosData);
      setTopics(topicsData);
      setLoading(false);
    };
    void load();
  }, []);

  const filtered = convos.filter((c) => {
    if (statusFilter !== 'all' && c.status !== statusFilter) {
      return false;
    }
    if (topicFilter !== 'all' && c.topic_id !== topicFilter) {
      return false;
    }
    return true;
  });

  const handleSelect = useCallback((id: string) => {
    router.push(`/chat/${id}`);
  }, [router]);

  if (!isAuthenticated) {
    return (
      <div className="container mx-auto px-4 py-20 max-w-md text-center">
        <h1 className="text-2xl font-bold mb-2">Chat History</h1>
        <p className="text-muted-foreground text-sm">Please sign in to access chat history.</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="container mx-auto px-4 py-20 max-w-md text-center">
        <Loader2 className="w-8 h-8 text-blue-400 animate-spin mx-auto mb-4" />
        <p className="text-sm text-muted-foreground">Loading history...</p>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8 max-w-3xl">
      {/* Header */}
      <div className="flex items-center gap-3 mb-6">
        <Link
          href="/chat"
          className="w-9 h-9 rounded-xl hover:bg-slate-100 dark:hover:bg-gray-100 dark:bg-slate-800/50 flex items-center justify-center transition-colors border border-slate-200 dark:border-white/8"
        >
          <ArrowLeft className="w-4 h-4" />
        </Link>
        <div>
          <h1 className="text-xl font-bold tracking-tight">Chat History</h1>
          <p className="text-xs text-muted-foreground/60">
            {convos.length} total conversation{convos.length !== 1 ? 's' : ''}
          </p>
        </div>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-2 mb-4 p-2.5 rounded-2xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700">
        <SlidersHorizontal className="w-3.5 h-3.5 text-muted-foreground/50 shrink-0 ml-1" />
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="flex-1 px-3 py-2 rounded-xl border border-slate-200 dark:border-white/8 bg-white dark:bg-slate-800/40 text-xs font-medium focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500/30 transition-all cursor-pointer"
        >
          <option value="all">All Statuses</option>
          <option value="open">Open</option>
          <option value="in_progress">In Progress</option>
          <option value="resolved">Resolved</option>
          <option value="closed">Closed</option>
        </select>
        <select
          value={topicFilter}
          onChange={(e) => setTopicFilter(e.target.value)}
          className="flex-1 px-3 py-2 rounded-xl border border-slate-200 dark:border-white/8 bg-white dark:bg-slate-800/40 text-xs font-medium focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500/30 transition-all cursor-pointer"
        >
          <option value="all">All Topics</option>
          {topics.map((t) => (
            <option key={t.id} value={t.id}>
              {t.label}
            </option>
          ))}
        </select>
      </div>

      {/* Results */}
      {filtered.length === 0 && (
        <div className="text-center py-20">
          <div className="w-14 h-14 rounded-2xl bg-slate-100 dark:bg-slate-800/40 flex items-center justify-center mx-auto mb-4 border border-slate-200 dark:border-slate-700">
            <Inbox className="w-7 h-7 text-muted-foreground/30" />
          </div>
          <p className="text-sm font-medium text-muted-foreground mb-1">No conversations found</p>
          <p className="text-xs text-muted-foreground/50">Try adjusting your filters</p>
        </div>
      )}

      {filtered.length > 0 && (
        <div className="bg-white dark:bg-slate-900/80 border border-slate-200 dark:border-slate-700 rounded-3xl overflow-hidden shadow-sm">
          {filtered.map((convo, i) => {
            const topic = topics.find((t) => t.id === convo.topic_id);
            const timeAgo = formatDistanceToNow(new Date(convo.last_message_at), { addSuffix: true });
            const hasUnread = convo.unread_user > 0;
            return (
              <button
                key={convo.id}
                onClick={() => handleSelect(convo.id)}
                className={`group w-full px-5 py-4 hover:bg-slate-50 dark:hover:bg-gray-100 dark:bg-slate-800/30 transition-all text-left ${
                  hasUnread ? 'bg-blue-50 dark:bg-blue-500/5 border-l-2 border-l-blue-500' : ''
                } ${i < filtered.length - 1 ? 'border-b border-slate-200 dark:border-slate-700' : ''}`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="flex-1 min-w-0">
                    <h3 className={`text-sm mb-2 truncate leading-tight ${hasUnread ? 'font-bold' : 'font-medium text-foreground/80'}`}>
                      {convo.subject}
                    </h3>
                    <div className="flex items-center gap-2 flex-wrap">
                      {topic && (
                        <span className="text-[10px] font-semibold text-muted-foreground bg-muted/50 px-2 py-0.5 rounded-md border border-border/30">
                          {topic.label}
                        </span>
                      )}
                      <ChatStatusBadge status={convo.status} />
                      <span className="text-[10px] text-muted-foreground/50">{timeAgo}</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 shrink-0 mt-0.5">
                    {hasUnread && (
                      <span className="min-w-[22px] h-[22px] px-1.5 rounded-full bg-blue-600 text-white text-[10px] flex items-center justify-center font-bold shadow-sm shadow-blue-500/20">
                        {convo.unread_user > 9 ? '9+' : convo.unread_user}
                      </span>
                    )}
                    <ChevronRight className="w-4 h-4 text-muted-foreground/20 group-hover:text-muted-foreground/50 group-hover:translate-x-0.5 transition-all" />
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
