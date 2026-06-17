import { cookies } from 'next/headers';
import { getChatInboxAction } from '@/app/actions/chat';
import { ChatInbox } from '@/components/chat/chat-inbox';
import { AuthBanner } from '@/shared/components/auth/auth-banner';
import { COOKIES, getServerAuthToken } from '@/shared/auth/cookies';
import { Headset } from 'lucide-react';

export default async function ChatPage() {
  const cookieStore = await cookies();
  const token = getServerAuthToken(cookieStore);

  if (!token) {
    return (
      <div className="container mx-auto px-4 py-12 max-w-xl">
        <div className="flex items-center gap-4 mb-8">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] flex items-center justify-center shadow-lg shadow-[#7645d9]/25">
            <Headset className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-xl font-bold tracking-tight">Support Center</h1>
            <p className="text-xs text-muted-foreground mt-0.5">Get help from our team · Usually replies in minutes</p>
          </div>
        </div>
        <AuthBanner
          message="Sign in to access Support Chat"
          description="Connect your wallet to start a conversation with our team"
          buttonText="Sign In to Chat"
        />
      </div>
    );
  }

  let userAddr: string | undefined;
  try {
    const raw = cookieStore.get(COOKIES.user)?.value;
    if (raw) {
      const user = JSON.parse(raw) as { wallet_address?: string };
      userAddr = user.wallet_address;
    }
  } catch { /* ignore */ }

  const inbox = await getChatInboxAction();

  return (
    <div className="fixed top-14 inset-x-0 bottom-0 overflow-hidden">
      <ChatInbox topics={inbox.topics} initConvos={inbox.conversations} userAddr={userAddr} />
    </div>
  );
}
