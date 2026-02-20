interface StatusBadgeProps {
  status: 'open' | 'in_progress' | 'resolved' | 'closed';
}

const STATUS_CONFIG = {
  open: { label: 'Open', dot: 'bg-amber-400', bg: 'bg-amber-400/10', text: 'text-amber-400' },
  in_progress: { label: 'In Progress', dot: 'bg-blue-400', bg: 'bg-blue-400/10', text: 'text-blue-400' },
  resolved: { label: 'Resolved', dot: 'bg-emerald-400', bg: 'bg-emerald-400/10', text: 'text-emerald-400' },
  closed: { label: 'Closed', dot: 'bg-zinc-400', bg: 'bg-zinc-400/10', text: 'text-zinc-400' },
} as const;

export function ChatStatusBadge({ status }: StatusBadgeProps) {
  const cfg = STATUS_CONFIG[status];
  return (
    <span className={`inline-flex items-center gap-1.5 text-[10px] font-semibold px-2 py-0.5 rounded-full ${cfg.bg} ${cfg.text}`}>
      <span className={`w-1.5 h-1.5 rounded-full ${cfg.dot}`} />
      {cfg.label}
    </span>
  );
}
