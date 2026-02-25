import { MessageCircle, Clock, CheckCircle, UserX } from 'lucide-react';
import type { ChatStats } from '@/shared/api/chat';

interface Props {
  stats: ChatStats;
}

export function ChatStatsPanel({ stats }: Props) {
  const total = stats.total_open + stats.total_in_progress + stats.total_resolved;
  const cards = [
    {
      label: 'Open',
      value: stats.total_open,
      icon: MessageCircle,
      accent: 'from-amber-500 to-orange-500',
      iconBg: 'bg-amber-500/15',
      iconColor: 'text-amber-400',
      borderColor: 'border-amber-500/10',
    },
    {
      label: 'In Progress',
      value: stats.total_in_progress,
      icon: Clock,
      accent: 'from-cyan-500 to-blue-500',
      iconBg: 'bg-cyan-500/15',
      iconColor: 'text-cyan-400',
      borderColor: 'border-cyan-500/10',
    },
    {
      label: 'Resolved',
      value: stats.total_resolved,
      icon: CheckCircle,
      accent: 'from-emerald-500 to-green-500',
      iconBg: 'bg-emerald-500/15',
      iconColor: 'text-emerald-400',
      borderColor: 'border-emerald-500/10',
    },
    {
      label: 'Unassigned',
      value: stats.total_unassigned,
      icon: UserX,
      accent: 'from-rose-500 to-red-500',
      iconBg: 'bg-rose-500/15',
      iconColor: 'text-rose-400',
      borderColor: 'border-rose-500/10',
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-3 mb-6">
      {cards.map(card => {
        const Icon = card.icon;
        return (
          <div
            key={card.label}
            className={`relative overflow-hidden bg-card border border-border/20 rounded-xl p-4 group hover:bg-muted/20 transition-colors`}
          >
            {/* Subtle gradient accent line */}
            <div className={`absolute top-0 left-0 right-0 h-0.5 bg-gradient-to-r ${card.accent} opacity-60`} />
            <div className="flex items-center justify-between mb-3">
              <div className={`p-2 rounded-xl ${card.iconBg}`}>
                <Icon className={`w-4.5 h-4.5 ${card.iconColor}`} />
              </div>
            </div>
            <p className="text-3xl font-black text-foreground tracking-tight">{card.value}</p>
            <p className="text-xs font-medium text-muted-foreground mt-1 uppercase tracking-wider">{card.label}</p>
          </div>
        );
      })}
    </div>
  );
}
