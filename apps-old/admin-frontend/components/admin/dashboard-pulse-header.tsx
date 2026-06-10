import type { DashboardStats } from '@/hooks/use-dashboard-data';
import { Activity } from 'lucide-react';
import { useEffect, useState } from 'react';

interface DashboardPulseHeaderProps {
    stats: DashboardStats;
}

export function DashboardPulseHeader({ stats }: DashboardPulseHeaderProps) {
    const [time, setTime] = useState(new Date());

    useEffect(() => {
        const timer = setInterval(() => setTime(new Date()), 1000);
        return () => clearInterval(timer);
    }, []);

    const isHealthy = stats.systemHealth >= 90;

    return (
        <div className="relative overflow-hidden rounded-2xl border border-border/20 bg-card/80 backdrop-blur-xl mb-6 shadow-2xl">
            {/* Background animated elements */}
            <div className="absolute inset-0 bg-gradient-to-r from-blue-500/5 via-purple-500/5 to-cyan-500/5" />
            <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-cyan-500/50 to-transparent opacity-50" />

            <div className="relative p-6 sm:p-8 flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
                <div>
                    <div className="flex items-center gap-3 mb-2">
                        <h1 className="text-3xl sm:text-4xl font-black text-foreground tracking-tight">
                            Command Center
                        </h1>
                        <div className={`px-3 py-1 rounded-full border text-xs font-bold uppercase tracking-wider flex items-center gap-2 ${isHealthy ? 'bg-success/10 text-success border-success/20' : 'bg-destructive/10 text-destructive border-destructive/20'
                            }`}>
                            <span className="relative flex h-2 w-2">
                                <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${isHealthy ? 'bg-success' : 'bg-destructive'
                                    }`} />
                                <span className={`relative inline-flex rounded-full h-2 w-2 ${isHealthy ? 'bg-success' : 'bg-destructive'
                                    }`} />
                            </span>
                            {isHealthy ? 'OPERATIONAL' : 'SYSTEM DEGRADED'}
                        </div>
                    </div>
                    <div className="text-muted-foreground font-mono text-sm flex items-center gap-4">
                        <span>{time.toISOString().split('T')[0]} {time.toLocaleTimeString()}</span>
                        <span className="text-border/50">|</span>
                        <span className="flex items-center gap-1.5 object-contain">
                            <Activity className="w-3.5 h-3.5 text-cyan-400" /> Pulse Active
                        </span>
                    </div>
                </div>

                <div className="flex shrink-0 items-center divide-x divide-border/30 rounded-xl border border-border/20 bg-background/50 p-2 backdrop-blur-md">
                    <div className="px-4 py-1 text-center">
                        <div className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground mb-1">Latency</div>
                        <div className="font-mono font-bold text-cyan-400">{stats.avgResponseTime}</div>
                    </div>
                    <div className="px-4 py-1 text-center">
                        <div className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground mb-1">Uptime</div>
                        <div className="font-mono font-bold text-primary">{stats.systemUptime}</div>
                    </div>
                    {stats.pendingNotifications > 0 && (
                        <div className="px-4 py-1 text-center bg-destructive/10 animate-pulse rounded-r-lg">
                            <div className="text-[10px] font-bold uppercase tracking-widest text-destructive mb-1">Alerts</div>
                            <div className="font-mono font-bold text-destructive">{stats.pendingNotifications}</div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
