import { Clock, Plus, Users } from 'lucide-react';
import Link from 'next/link';

import { type PolicyStats } from '../types';

interface QuickActionsProps {
    stats: PolicyStats;
}

export function QuickActions({ stats }: QuickActionsProps) {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            {/* Create Plan */}
            <Link href="/wallet-management/access/plans" className="block group h-full">
                <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/20 via-indigo-400/20 to-blue-400/20 p-0.5 hover:scale-[1.02] transition-all duration-300 h-full">
                    <div className="relative bg-card rounded-2xl h-full flex flex-col">
                        <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-indigo-500 rounded-full blur-sm opacity-60" />
                        <div className="p-4 sm:p-6 flex-1 flex flex-col">
                            <div className="flex items-center gap-2 mb-2">
                                <Plus className="w-5 h-5 text-blue-500" />
                                <h3 className="text-lg font-bold bg-gradient-to-r from-blue-400 to-indigo-500 bg-clip-text text-transparent">
                                    New Plan
                                </h3>
                            </div>
                            <p className="text-sm text-muted-foreground mb-3">
                                Create a subscription plan
                            </p>
                            <div className="flex items-center justify-between mt-auto">
                                <div className="px-3 py-1 bg-gradient-to-r from-blue-400 to-indigo-500 text-white rounded-full text-xs font-medium">
                                    💳 Plan
                                </div>
                                <div className="text-muted-foreground group-hover:translate-x-1 transition-transform duration-200">
                                    →
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Link>

            {/* Create Group */}
            <Link
                href="/wallet-management/groups/new"
                className="block group h-full"
            >
                <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-amber-400/20 via-orange-400/20 to-amber-400/20 p-0.5 hover:scale-[1.02] transition-all duration-300 h-full">
                    <div className="relative bg-card rounded-2xl h-full flex flex-col">
                        <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-amber-400 to-orange-500 rounded-full blur-sm opacity-60" />
                        <div className="p-4 sm:p-6 flex-1 flex flex-col">
                            <div className="flex items-center gap-2 mb-2">
                                <Users className="w-5 h-5 text-amber-500" />
                                <h3 className="text-lg font-bold bg-gradient-to-r from-amber-400 to-orange-500 bg-clip-text text-transparent">
                                    New Group
                                </h3>
                            </div>
                            <p className="text-sm text-muted-foreground mb-3">
                                Create a manual access group
                            </p>
                            <div className="flex items-center justify-between mt-auto">
                                <div className="px-3 py-1 bg-gradient-to-r from-amber-400 to-orange-500 text-white rounded-full text-xs font-medium">
                                    👥 Group
                                </div>
                                <div className="text-muted-foreground group-hover:translate-x-1 transition-transform duration-200">
                                    →
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Link>

            {/* View Expiring */}
            <Link
                href="/wallet-management/wallets"
                className="block group h-full"
            >
                <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-orange-400/20 via-pink-400/20 to-orange-400/20 p-0.5 hover:scale-[1.02] transition-all duration-300 h-full">
                    <div className="relative bg-card rounded-2xl h-full flex flex-col">
                        <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-orange-400 to-pink-500 rounded-full blur-sm opacity-60" />
                        <div className="p-4 sm:p-6 flex-1 flex flex-col">
                            <div className="flex items-center gap-2 mb-2">
                                <Clock className="w-5 h-5 text-orange-500" />
                                <h3 className="text-lg font-bold bg-gradient-to-r from-orange-400 to-pink-500 bg-clip-text text-transparent">
                                    Expiring Soon
                                </h3>
                            </div>
                            <p className="text-sm text-muted-foreground mb-3">
                                {stats.expiringSoon > 0
                                    ? `${stats.expiringSoon} expiring within 7 days`
                                    : 'View expiring access'}
                            </p>
                            <div className="flex items-center justify-between mt-auto">
                                <div className="px-3 py-1 bg-gradient-to-r from-orange-400 to-pink-500 text-white rounded-full text-xs font-medium">
                                    {stats.expiringSoon > 0 ? stats.expiringSoon : 'View'}
                                </div>
                                <div className="text-muted-foreground group-hover:translate-x-1 transition-transform duration-200">
                                    →
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Link>
        </div>
    );
}
