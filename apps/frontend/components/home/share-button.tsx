'use client';

import { Button } from '@/components/ui/button';
import { copyToClipboard } from '@/utils/clipboard';
import { Share2 } from 'lucide-react';
import { toast } from 'sonner';

export function ShareButton() {
  const handleShare = async () => {
    if (typeof window !== 'undefined') {
      const success = await copyToClipboard(window.location.href);
      if (success) { toast.success('URL copied to clipboard!'); }
    }
  };

  return (
    <Button
      onClick={handleShare}
      className="w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-2 border-orange-400/50 rounded-2xl shadow-xl hover:shadow-orange-300/30 hover:scale-105 transition-all duration-300 group"
    >
      <Share2 className="mr-3 h-6 w-6 group-hover:animate-wiggle" />
      📤 Share Platform
    </Button>
  );
}
