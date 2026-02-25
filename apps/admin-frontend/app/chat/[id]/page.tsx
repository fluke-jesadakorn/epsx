import { Breadcrumb } from '@/components/layout/breadcrumb';
import { ChatConversationView } from '@/components/chat';
import { getConversation, getTopics } from '@/app/actions/chat';
import { notFound } from 'next/navigation';

interface Props {
  params: Promise<{ id: string }>;
}

export default async function ConversationPage({ params }: Props) {
  const { id } = await params;

  const [convRes, topicsRes] = await Promise.all([
    getConversation(id),
    getTopics(),
  ]);

  if (!convRes.success || !convRes.data) {
    notFound();
  }

  const conv = convRes.data;
  const topics = topicsRes.success && topicsRes.data ? topicsRes.data : [];

  return (
    <div className="p-4 md:p-8">
      <Breadcrumb />
      <div className="h-[calc(100vh-7rem)] md:h-[calc(100vh-10rem)] border border-border/20 rounded-2xl bg-gray-100 dark:bg-card/80 overflow-hidden">
        <ChatConversationView
          conv={conv}
          topics={topics}
          onUpdate={() => {
            // Reload page on update
            window.location.reload();
          }}
        />
      </div>
    </div>
  );
}
