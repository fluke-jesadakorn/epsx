import type { Tier } from './data/endpoints';

const tierStyles: Record<Tier, string> = {
  free: 'bg-cyan-500/10 text-cyan-400 border-cyan-500/20',
  basic: 'bg-green-500/10 text-green-400 border-green-500/20',
  premium: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
  enterprise: 'bg-orange-500/10 text-orange-400 border-orange-500/20',
};

export function TierBadge({ tier }: { tier: Tier }) {
  return (
    <span className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium capitalize ${tierStyles[tier]}`}>
      {tier}
    </span>
  );
}
