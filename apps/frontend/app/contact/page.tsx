import { Card, CardContent } from '@/components/ui';
import { Mail, MessageSquare, Clock, Shield } from 'lucide-react';
import { ContactForm, CopyEmailBtn } from './contact-form';

const SUPPORT_EMAIL = 'support@epsx.io';

const INFO_CARDS = [
  {
    icon: MessageSquare,
    title: 'General Inquiries',
    desc: 'Questions about our platform, features, or pricing plans.',
    color: 'from-purple-500 to-blue-500',
    border: 'border-purple-200/50 dark:border-purple-400/20',
  },
  {
    icon: Shield,
    title: 'Technical Support',
    desc: 'Need help with your account, API access, or integrations.',
    color: 'from-orange-500 to-yellow-500',
    border: 'border-orange-200/50 dark:border-orange-400/20',
  },
  {
    icon: Clock,
    title: 'Response Time',
    desc: 'We typically respond within 24 hours on business days.',
    color: 'from-blue-500 to-cyan-500',
    border: 'border-blue-200/50 dark:border-blue-400/20',
  },
];

export default function ContactPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background */}
      <div className="fixed inset-0 z-0">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />
        <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-purple-400/30 to-pink-400/30 blur-3xl" />
        <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-orange-400/25 to-yellow-400/25 blur-3xl" />
        <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-3xl" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(168,85,247,0.1)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(255,133,27,0.08)_0%,_transparent_50%)]" />
      </div>

      {/* Content */}
      <div className="relative z-10">
        {/* Header */}
        <div className="container mx-auto px-4 pt-16 pb-8">
          <div className="text-center">
            <h1 className="text-5xl sm:text-6xl font-bold bg-gradient-to-r from-purple-500 via-orange-500 to-yellow-500 dark:from-purple-400 dark:via-orange-400 dark:to-yellow-400 bg-clip-text text-transparent mb-4">
              Contact Us
            </h1>
            <p className="mx-auto max-w-2xl text-lg leading-relaxed text-gray-600 dark:text-gray-300">
              Have a question or need support? We&apos;d love to hear from you.
            </p>
            <div className="w-40 h-1 bg-gradient-to-r from-purple-500 via-orange-500 to-yellow-500 mx-auto rounded-full mt-6" />
          </div>
        </div>

        {/* Email highlight */}
        <div className="container mx-auto px-4 pb-8">
          <div className="flex items-center justify-center">
            <div className="inline-flex items-center gap-3 bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border border-purple-200/50 dark:border-purple-400/20 rounded-2xl px-6 py-3 shadow-lg">
              <Mail className="h-5 w-5 text-purple-500" />
              <a
                href={`mailto:${SUPPORT_EMAIL}`}
                className="text-lg font-semibold text-gray-800 dark:text-gray-100 hover:text-purple-500 dark:hover:text-purple-400 transition-colors"
              >
                {SUPPORT_EMAIL}
              </a>
              <CopyEmailBtn />
            </div>
          </div>
        </div>

        {/* Main content grid */}
        <div className="container mx-auto px-4 pb-16">
          <div className="grid grid-cols-1 lg:grid-cols-5 gap-8 max-w-6xl mx-auto">
            {/* Contact Form */}
            <div className="lg:col-span-3">
              <Card className="bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border border-gray-200/50 dark:border-slate-600/30 rounded-3xl shadow-2xl overflow-hidden">
                <CardContent className="p-8">
                  <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">
                    Send us a message
                  </h2>
                  <ContactForm />
                </CardContent>
              </Card>
            </div>

            {/* Info cards */}
            <div className="lg:col-span-2 space-y-4">
              {INFO_CARDS.map(c => (
                <Card
                  key={c.title}
                  className={`bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border ${c.border} rounded-3xl shadow-xl`}
                >
                  <CardContent className="p-6">
                    <div className="flex items-start gap-4">
                      <div className={`p-3 rounded-2xl bg-gradient-to-br ${c.color} shrink-0`}>
                        <c.icon className="h-5 w-5 text-white" />
                      </div>
                      <div>
                        <h3 className="font-semibold text-gray-800 dark:text-gray-100 mb-1">
                          {c.title}
                        </h3>
                        <p className="text-sm text-gray-500 dark:text-gray-400">
                          {c.desc}
                        </p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
