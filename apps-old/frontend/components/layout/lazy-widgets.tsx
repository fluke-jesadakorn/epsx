'use client';

import dynamic from 'next/dynamic';

export const ChatWidget = dynamic(
  () => import('@/components/chat/chat-widget').then(m => m.ChatWidget),
  { ssr: false },
);

export const FrontendAuthModal = dynamic(
  () => import('@/components/auth/frontend-auth-modal').then(m => m.FrontendAuthModal),
  { ssr: false },
);
