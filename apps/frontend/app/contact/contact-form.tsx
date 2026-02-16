'use client';

import { useState } from 'react';
import { Button, Input, Label, Textarea } from '@/components/ui';
import { Copy, Check, Send } from 'lucide-react';

const SUPPORT_EMAIL = 'support@epsx.io';

interface FormData {
  name: string;
  email: string;
  subject: string;
  message: string;
}

export function CopyEmailBtn() {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    void navigator.clipboard.writeText(SUPPORT_EMAIL);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={copy}
      className="inline-flex items-center gap-1.5 text-sm text-purple-500 hover:text-purple-400 transition-colors"
    >
      {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
      {copied ? 'Copied!' : 'Copy'}
    </button>
  );
}

export function ContactForm() {
  const [form, setForm] = useState<FormData>({ name: '', email: '', subject: '', message: '' });

  const set = (field: keyof FormData) => (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
    setForm(prev => ({ ...prev, [field]: e.target.value }));

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const subj = encodeURIComponent(form.subject || 'Contact from EPSX');
    const body = encodeURIComponent(
      `Name: ${form.name}\nEmail: ${form.email}\n\n${form.message}`
    );
    window.open(`mailto:${SUPPORT_EMAIL}?subject=${subj}&body=${body}`, '_self');
  };

  const filled = form.name.trim() !== '' && form.email.trim() !== '' && form.message.trim() !== '';

  return (
    <form onSubmit={submit} className="space-y-5">
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label htmlFor="name" className="text-gray-700 dark:text-gray-300">Name</Label>
          <Input
            id="name"
            placeholder="Your name"
            value={form.name}
            onChange={set('name')}
            className="bg-white/50 dark:bg-slate-700/50 border-gray-200 dark:border-slate-600"
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="email" className="text-gray-700 dark:text-gray-300">Email</Label>
          <Input
            id="email"
            type="email"
            placeholder="your@email.com"
            value={form.email}
            onChange={set('email')}
            className="bg-white/50 dark:bg-slate-700/50 border-gray-200 dark:border-slate-600"
          />
        </div>
      </div>
      <div className="space-y-2">
        <Label htmlFor="subject" className="text-gray-700 dark:text-gray-300">Subject</Label>
        <Input
          id="subject"
          placeholder="How can we help?"
          value={form.subject}
          onChange={set('subject')}
          className="bg-white/50 dark:bg-slate-700/50 border-gray-200 dark:border-slate-600"
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="message" className="text-gray-700 dark:text-gray-300">Message</Label>
        <Textarea
          id="message"
          placeholder="Tell us more about your inquiry..."
          value={form.message}
          onChange={set('message')}
          rows={5}
          className="bg-white/50 dark:bg-slate-700/50 border-gray-200 dark:border-slate-600 resize-none"
        />
      </div>
      <Button
        type="submit"
        disabled={!filled}
        className="w-full bg-gradient-to-r from-purple-500 to-orange-500 hover:from-purple-600 hover:to-orange-600 text-white font-semibold py-3 rounded-xl disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <Send className="h-4 w-4 mr-2" />
        Send Message
      </Button>
      <p className="text-xs text-center text-gray-400 dark:text-gray-500">
        This will open your default email client
      </p>
    </form>
  );
}
