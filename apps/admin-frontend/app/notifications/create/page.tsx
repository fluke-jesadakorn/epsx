'use client';

import { SendNotificationForm } from '@/components/notifications/send-notification-form';
import { useRouter } from 'next/navigation';

export default function CreateNotificationPage() {
    const router = useRouter();

    return (
        <div className="relative overflow-hidden rounded-[40px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 p-1 shadow-2xl">
            <div className="relative bg-card/60 backdrop-blur-md rounded-[38px] p-8 sm:p-12">
                <div className="max-w-4xl mx-auto">
                    <div className="mb-12">
                        <h2 className="text-3xl font-black text-foreground uppercase tracking-tight mb-2">
                            Signal Generator
                        </h2>
                        <p className="text-sm font-bold text-muted-foreground">Construct and transmit high-priority system alerts</p>
                    </div>
                    <SendNotificationForm
                        onSuccess={() => router.push('/notifications/manage')}
                        onCancel={() => router.push('/notifications/manage')}
                    />
                </div>
            </div>
        </div>
    );
}
