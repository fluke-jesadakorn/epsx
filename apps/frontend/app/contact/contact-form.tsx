'use client';

import { copyToClipboard } from '@/utils/clipboard';
import { Check, Copy, Mail } from 'lucide-react';
import { useState } from 'react';

const SUPPORT_EMAIL = 'info@epsx.io';

export function MailtoBtn({ className }: { className?: string }) {
  return (
    <button
      type="button"
      onClick={() => { window.location.href = `mailto:${SUPPORT_EMAIL}`; }}
      className={className}
    >
      <Mail className="h-4 w-4" />
      {SUPPORT_EMAIL}
    </button>
  );
}

export function CopyEmailBtn() {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    const ok = await copyToClipboard(SUPPORT_EMAIL);
    if (ok) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <button
      type="button"
      onClick={() => { void copy(); }}
      className="inline-flex items-center gap-1.5 text-sm text-purple-500 hover:text-purple-400 transition-colors"
    >
      {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
      {copied ? 'Copied!' : 'Copy'}
    </button>
  );
}
