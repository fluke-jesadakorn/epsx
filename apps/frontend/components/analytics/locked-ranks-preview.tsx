interface LockedRanksPreviewProps {
  minAccessibleRank: number;  // User's minimum accessible rank
  lockedCount: number;         // How many ranks are locked
}

/**
 *
 * @param root0
 * @param root0.minAccessibleRank
 * @param root0.lockedCount
 */
export function LockedRanksPreview({ minAccessibleRank, lockedCount }: LockedRanksPreviewProps): React.JSX.Element | null {
  if (minAccessibleRank <= 1 || lockedCount === 0) {
    return null; // User has top access, no locked ranks
  }

  // Show preview of 5 locked ranks just above user's access
  const previewRanks = Array.from(
    { length: Math.min(5, minAccessibleRank - 1) },
    (_, i) => minAccessibleRank - 5 + i + 1
  ).filter(r => r > 0);

  return (
    <div className="mb-8 border-2 border-orange-400 rounded-2xl p-6 bg-gradient-to-br from-orange-50 to-yellow-50 dark:from-orange-950/20 dark:to-yellow-950/20">
      <div className="text-center mb-6">
        <div className="text-4xl mb-2">🔒</div>
        <h3 className="text-2xl font-bold text-orange-800 dark:text-orange-300 mb-2">
          {lockedCount} Premium Ranks Locked
        </h3>
        <p className="text-orange-600 dark:text-orange-400">
          You can see ranks from #{minAccessibleRank} onwards
        </p>
      </div>

      {/* Blurred preview cards */}
      <div className="flex gap-3 overflow-x-auto mb-6 pb-2">
        {previewRanks.map(rank => (
          <div key={rank} className="relative min-w-[180px] h-[280px] rounded-xl overflow-hidden flex-shrink-0 shadow-lg">
            {/* Blurred background */}
            <div className="absolute inset-0 bg-gradient-to-br from-slate-200 to-slate-300 dark:from-slate-700 dark:to-slate-800" />
            <div className="absolute inset-0 backdrop-blur-xl bg-white/30 dark:bg-black/30" />

            {/* Lock icon overlay */}
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <div className="text-5xl mb-3">🔒</div>
              <div className="text-2xl font-bold text-slate-700 dark:text-slate-200">Rank #{rank}</div>
              <div className="text-sm text-slate-500 dark:text-slate-400 mt-1">Premium Only</div>
            </div>
          </div>
        ))}
      </div>

      {/* Upgrade CTA */}
      <button
        onClick={() => window.location.href = '/plans'}
        className="w-full bg-gradient-to-r from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 text-white py-4 rounded-xl font-bold text-lg shadow-lg"
      >
        🚀 Unlock Ranks 1-{minAccessibleRank - 1} → Upgrade to Premium
      </button>

      <div className="text-center mt-3 text-sm text-orange-700 dark:text-orange-400">
        Get access to top {minAccessibleRank - 1} performing stocks
      </div>
    </div>
  );
}
