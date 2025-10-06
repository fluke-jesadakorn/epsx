interface AccessBannerProps {
  minRank: number;  // Minimum rank user can access
}

/**
 *
 * @param root0
 * @param root0.minRank
 */
export function AccessBanner({ minRank }: AccessBannerProps): React.JSX.Element {
  const accessLevel =
    minRank === 1 ? { level: '👑 Ultimate Access', color: 'purple', bgColor: 'bg-purple-50 dark:bg-purple-950/20', borderColor: 'border-purple-500', textColor: 'text-purple-900 dark:text-purple-200', btnColor: 'bg-purple-500 hover:bg-purple-600' } :
    minRank <= 25 ? { level: '⭐ Premium Access', color: 'blue', bgColor: 'bg-blue-50 dark:bg-blue-950/20', borderColor: 'border-blue-500', textColor: 'text-blue-900 dark:text-blue-200', btnColor: 'bg-blue-500 hover:bg-blue-600' } :
    minRank <= 50 ? { level: '💎 Advanced Access', color: 'green', bgColor: 'bg-green-50 dark:bg-green-950/20', borderColor: 'border-green-500', textColor: 'text-green-900 dark:text-green-200', btnColor: 'bg-green-500 hover:bg-green-600' } :
    minRank <= 75 ? { level: '🥈 Standard Access', color: 'orange', bgColor: 'bg-orange-50 dark:bg-orange-950/20', borderColor: 'border-orange-500', textColor: 'text-orange-900 dark:text-orange-200', btnColor: 'bg-orange-500 hover:bg-orange-600' } :
    { level: '📊 Basic Access', color: 'gray', bgColor: 'bg-gray-50 dark:bg-gray-950/20', borderColor: 'border-gray-500', textColor: 'text-gray-900 dark:text-gray-200', btnColor: 'bg-gray-500 hover:bg-gray-600' };

  return (
    <div className={`${accessLevel.bgColor} border-l-4 ${accessLevel.borderColor} p-4 mb-6 rounded-r-lg`}>
      <div className="flex items-center justify-between">
        <div>
          <div className={`font-bold text-lg ${accessLevel.textColor}`}>
            {accessLevel.level}
          </div>
          <div className="text-sm text-slate-600 dark:text-slate-400">
            {minRank === 1 ? (
              <>Viewing from <span className="font-semibold">Top Rank #1</span> onwards</>
            ) : (
              <>Viewing from <span className="font-semibold">Rank #{minRank}</span> onwards</>
            )}
          </div>
        </div>

        {minRank > 1 && (
          <button
            onClick={() => window.location.href = '/plans'}
            className={`${accessLevel.btnColor} text-white px-6 py-2 rounded-lg font-semibold shadow-md`}
          >
            Unlock Top {minRank - 1} →
          </button>
        )}
      </div>
    </div>
  );
}
