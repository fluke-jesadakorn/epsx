import { CheckCircle, Clock, AlertCircle, XCircle } from 'lucide-react';

interface Props {
  status: 'open' | 'in_progress' | 'resolved' | 'closed';
}

export function ChatStatusBadge({ status }: Props) {
  const cfg = {
    open: {
      label: 'Open',
      color: 'text-amber-400 bg-amber-500/10 border-amber-500/20',
      icon: Clock,
    },
    in_progress: {
      label: 'In Progress',
      color: 'text-cyan-400 bg-cyan-500/10 border-cyan-500/20',
      icon: AlertCircle,
    },
    resolved: {
      label: 'Resolved',
      color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
      icon: CheckCircle,
    },
    closed: {
      label: 'Closed',
      color: 'text-zinc-400 bg-zinc-500/10 border-zinc-500/20',
      icon: XCircle,
    },
  }[status];

  const Icon = cfg.icon;

  return (
    <div className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-bold border ${cfg.color} uppercase tracking-wide`}>
      <Icon className="w-3 h-3" />
      {cfg.label}
    </div>
  );
}
