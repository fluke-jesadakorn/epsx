'use client';

import { MessageCircle } from "lucide-react";
import { Button } from "@/components/ui";
import { lazy, Suspense, useState } from "react";

const ChatDialog = lazy(() => import('./ChatDialog').then(mod => ({ default: mod.ChatDialog })));

export function ChatTrigger() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      <div className="fixed bottom-8 right-8 z-50">
        <Button
          onClick={() => setIsOpen(true)}
          className="rounded-full w-14 h-14 bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 text-white shadow-lg transform hover:scale-105 transition-all duration-200"
        >
          <MessageCircle className="h-7 w-7" />
        </Button>
      </div>
      
      {isOpen && (
        <Suspense fallback={null}>
          <ChatDialog isOpen={isOpen} onClose={() => setIsOpen(false)} />
        </Suspense>
      )}
    </>
  );
}