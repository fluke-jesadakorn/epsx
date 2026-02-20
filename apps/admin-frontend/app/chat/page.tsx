import { Breadcrumb } from '@/components/layout/breadcrumb';
import { ChatInbox, ChatStatsPanel } from '@/components/chat';
import { getAdminChatStats, listAllConversations, getTopics } from '@/app/actions/chat';
import { MessageCircle } from 'lucide-react';

export default async function ChatPage() {
  const [statsRes, convsRes, topicsRes] = await Promise.all([
    getAdminChatStats(),
    listAllConversations(),
    getTopics(),
  ]);

  const stats = statsRes.success && statsRes.data ? statsRes.data : {
    total_open: 0,
    total_in_progress: 0,
    total_resolved: 0,
    total_unassigned: 0,
  };

  const convs = convsRes.success && convsRes.data ? convsRes.data : [];
  const topics = topicsRes.success && topicsRes.data ? topicsRes.data : [];

  return (
    <div className="p-8">
      <Breadcrumb />
      <div className="flex items-center gap-3 mb-6">
        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center shadow-sm shadow-violet-500/20">
          <MessageCircle className="w-5 h-5 text-white" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-foreground tracking-tight">Chat Support</h1>
          <p className="text-xs text-muted-foreground/60">Manage support conversations</p>
        </div>
      </div>
      <ChatStatsPanel stats={stats} />
      <ChatInbox initConvs={convs} topics={topics} />
    </div>
  );
}
