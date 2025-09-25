import {
  calculateEstimatedGrowth,
  calculateDaysRemaining,
  calculatePERatio,
  calculatePriceGrowth,
  calculatePriceTarget,
  formatCountdown,
  getGrowthIndicator,
  formatPercentage,
  formatCurrency,
  formatEPS,
  formatPERatio,
  formatQuarterDate,
  formatAnnouncementDate,
  getQuarterLabel,
  formatTimeRemaining,
  formatCompactDate
} from '@/lib/consolidated-utils'

interface AdminEPSCardData {
  rank: number
  symbol: string
  latest_date: string
  value: number
  active_status: string
  quarterly_performance: {
    quarter: string
    date: string
    price: number
    eps: number
    eps_growth: number
    price_growth: number
    is_estimated?: boolean
  }[]
  next_quarter_estimate?: {
    quarter: string
    announcement_date: string
    days_until_announcement: number
    estimated_eps: number
    estimated_price_target?: number
    confidence: string
  }
  admin_data?: {
    system_alerts: number
  }
  // 4-Quarter EPS Data Structure (from backend)
  eps_quarterly?: {
    eps_q_minus_2?: number
    eps_q_minus_1?: number  
    eps_q_current?: number
    eps_q_next_estimate?: number
    eps_q_minus_2_date?: string
    eps_q_minus_1_date?: string
    eps_q_current_date?: string
    eps_q_next_estimate_date?: string
    qoq_growth_current?: number
    yoy_growth_current?: number
    trend_direction?: 'UP' | 'DOWN' | 'FLAT'
    avg_growth_rate?: number
    consistency_score?: 'HIGH' | 'MEDIUM' | 'LOW'
  }
}

interface AdminEPSCardProps {
  cardData: AdminEPSCardData
}




export default function AdminEPSCard({ cardData }: AdminEPSCardProps) {
  const quarters = cardData.quarterly_performance?.slice(0, 2) || []
  const currentQuarter = quarters[0] || {}
  const previousQuarter = quarters[1] || {}
  
  // Calculate enhanced data points with Visual Dashboard Style focus
  const currentPrice = currentQuarter.price || 0
  const currentEPS = currentQuarter.eps || 0
  const previousPrice = previousQuarter.price || 0
  const previousEPS = previousQuarter.eps || 0
  
  const currentPE = calculatePERatio(currentPrice, currentEPS)
  const priceGrowth = calculatePriceGrowth(currentPrice, previousPrice)
  const epsGrowth = currentQuarter.eps_growth || 0
  
  const estimatedEPS = cardData.eps_quarterly?.eps_q_next_estimate || cardData.next_quarter_estimate?.estimated_eps || 0
  const estimatedGrowth = calculateEstimatedGrowth(estimatedEPS, currentEPS)
  const estimatedPriceTarget = calculatePriceTarget(estimatedEPS, currentPE)
  
  const daysRemaining = cardData.next_quarter_estimate?.days_until_announcement || 0
  
  // Growth indicators for Visual Dashboard Style
  const priceGrowthIndicator = getGrowthIndicator(priceGrowth)
  const epsGrowthIndicator = getGrowthIndicator(epsGrowth)
  const estimatedGrowthIndicator = getGrowthIndicator(estimatedGrowth)
  
  // Quarterly evolution data for grid
  const quarterlyEvolution = [
    {
      label: 'Q-2',
      price: cardData.eps_quarterly?.eps_q_minus_2 ? (cardData.eps_quarterly.eps_q_minus_2 * currentPE) : previousPrice * 0.92,
      eps: cardData.eps_quarterly?.eps_q_minus_2 || previousEPS * 0.92,
      priceGrowth: -3.5,
      epsGrowth: null,
      isCurrent: false,
      isEstimate: false,
      date: cardData.eps_quarterly?.eps_q_minus_2_date
    },
    {
      label: 'Q-1',  
      price: previousPrice,
      eps: previousEPS,
      priceGrowth: -3.5,
      epsGrowth: 0,
      isCurrent: false,
      isEstimate: false,
      date: previousQuarter.date
    },
    {
      label: 'Q0',
      price: currentPrice,
      eps: currentEPS,
      priceGrowth: priceGrowth,
      epsGrowth: epsGrowth,
      isCurrent: true,
      isEstimate: false,
      date: currentQuarter.date
    },
    {
      label: 'Q+1',
      price: estimatedPriceTarget,
      eps: estimatedEPS,
      priceGrowth: estimatedPriceTarget ? calculatePriceGrowth(estimatedPriceTarget, currentPrice) : 0,
      epsGrowth: estimatedGrowth,
      isCurrent: false,
      isEstimate: true,
      date: cardData.next_quarter_estimate?.announcement_date
    }
  ]
  
  return (
    <div className={`hover:shadow-3xl relative w-full max-w-[420px] min-w-[360px] sm:min-w-[380px] flex-shrink-0 overflow-hidden rounded-3xl border-2 bg-gradient-to-br from-white via-slate-50 to-gray-100 dark:from-gray-800 dark:via-gray-750 dark:to-gray-700 shadow-2xl transition-all duration-300 hover:scale-105 ${
      cardData.active_status === 'TRACK'
        ? 'border-green-300 dark:border-green-600 shadow-green-500/20 dark:shadow-green-500/10 hover:shadow-green-500/30 dark:hover:shadow-green-500/20'
        : cardData.active_status === 'STOP'
          ? 'border-red-300 dark:border-red-600 shadow-red-500/20 dark:shadow-red-500/10 hover:shadow-red-500/30 dark:hover:shadow-red-500/20'
          : 'border-orange-300 dark:border-orange-600 shadow-orange-500/20 dark:shadow-orange-500/10 hover:shadow-orange-500/30 dark:hover:shadow-orange-500/20'
    }`}>
      
      {/* System alerts indicator */}
      {(cardData.admin_data?.system_alerts || 0) > 0 && (
        <div className="absolute top-2 left-2 h-6 w-6 bg-red-500 text-white text-xs font-bold rounded-full flex items-center justify-center animate-pulse z-20">
          {cardData.admin_data?.system_alerts || 0}
        </div>
      )}

      <div className="p-6">
        {/* Visual Dashboard Header */}
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-lg">🏆</span>
            <div>
              <div className="text-xs font-medium text-slate-500 dark:text-slate-400 uppercase">#{cardData.rank}</div>
              <h3 className="text-xl font-bold text-slate-800 dark:text-slate-100">{cardData.symbol}</h3>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <a
              href={`https://www.tradingview.com/symbols/${cardData.symbol}`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 font-medium"
            >
              📊 TradingView
            </a>
            <div className={`px-3 py-1 rounded-full text-xs font-semibold ${
              cardData.active_status === 'TRACK'
                ? 'bg-green-100 dark:bg-green-800/30 text-green-800 dark:text-green-300'
                : cardData.active_status === 'STOP'
                  ? 'bg-red-100 dark:bg-red-800/30 text-red-800 dark:text-red-300'
                  : 'bg-orange-100 dark:bg-orange-800/30 text-orange-800 dark:text-orange-300'
            }`}>
              {cardData.active_status === 'TRACK' ? '📈 ACTIVE' : cardData.active_status === 'STOP' ? '📉 INACTIVE' : `📊 ${cardData.active_status}`}
            </div>
          </div>
        </div>

        {/* Main Metrics Boxes */}
        <div className="grid grid-cols-3 gap-2 sm:gap-3 mb-4">
          {/* Price Box */}
          <div className="bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/30 p-4 rounded-xl border border-blue-200 dark:border-blue-700/50">
            <div className="text-center">
              <div className="text-xs text-blue-700 dark:text-blue-300 font-medium mb-1">PRICE</div>
              <div className="text-lg font-bold text-blue-900 dark:text-blue-100">{formatCurrency(currentPrice)}</div>
              <div className={`text-xs font-medium flex items-center justify-center gap-1 ${priceGrowthIndicator.color}`}>
                <span>{priceGrowthIndicator.emoji}</span>
                <span>{formatPercentage(priceGrowth)}</span>
              </div>
            </div>
          </div>

          {/* EPS Box */}
          <div className="bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/30 p-4 rounded-xl border border-green-200 dark:border-green-700/50">
            <div className="text-center">
              <div className="text-xs text-green-700 dark:text-green-300 font-medium mb-1">EPS</div>
              <div className="text-lg font-bold text-green-900 dark:text-green-100">{formatEPS(currentEPS)}</div>
              <div className={`text-xs font-medium flex items-center justify-center gap-1 ${epsGrowthIndicator.color}`}>
                <span>{epsGrowthIndicator.emoji}</span>
                <span>{formatPercentage(epsGrowth)}</span>
              </div>
            </div>
          </div>

          {/* P/E Box */}
          <div className="bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/30 p-4 rounded-xl border border-purple-200 dark:border-purple-700/50">
            <div className="text-center">
              <div className="text-xs text-purple-700 dark:text-purple-300 font-medium mb-1">P/E</div>
              <div className="text-lg font-bold text-purple-900 dark:text-purple-100">{formatPERatio(currentPE)}</div>
              <div className="text-xs text-purple-600 dark:text-purple-300">
                📊
              </div>
            </div>
          </div>
        </div>

        {/* Next Earnings Info Box */}
        {cardData.next_quarter_estimate && (
          <div className="bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/30 p-4 rounded-xl border border-orange-200 dark:border-orange-700/50 mb-4">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-xs text-orange-700 dark:text-orange-300 font-medium mb-1">NEXT EARNINGS</div>
                <div className="text-sm font-semibold text-orange-900 dark:text-orange-100">
                  📅 {formatAnnouncementDate(cardData.next_quarter_estimate.announcement_date)}
                </div>
                <div className="text-xs text-orange-700 dark:text-orange-300">⏰ {daysRemaining} days</div>
              </div>
              <div className="text-right">
                <div className="text-xs text-orange-700 dark:text-orange-300 font-medium mb-1">EST. EPS</div>
                <div className="text-lg font-bold text-orange-900 dark:text-orange-100">🎯 {formatEPS(estimatedEPS)}</div>
                <div className={`text-xs font-medium flex items-center justify-end gap-1 ${estimatedGrowthIndicator.color}`}>
                  <span>{estimatedGrowthIndicator.emoji}</span>
                  <span>{formatPercentage(estimatedGrowth)} growth</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Quarterly Evolution Grid */}
        <div className="mb-4">
          <div className="text-center mb-3">
            <div className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-wide">QUARTERLY EVOLUTION</div>
          </div>
          <div className="bg-white/70 dark:bg-gray-800/70 rounded-xl border border-slate-200 dark:border-slate-600 overflow-hidden">
            <div className="grid grid-cols-4 gap-0 text-xs sm:text-sm">
              {quarterlyEvolution.map((quarter, index) => (
                <div key={quarter.label} className={`relative p-3 border-r border-slate-200 dark:border-slate-600 last:border-r-0 ${quarter.isCurrent ? 'bg-blue-50 dark:bg-blue-900/30' : quarter.isEstimate ? 'bg-purple-50 dark:bg-purple-900/30' : 'bg-slate-50 dark:bg-slate-800/50'}`}>
                  {/* Markers */}
                  {quarter.isCurrent && <div className="absolute top-1 right-1 w-2 h-2 bg-blue-500 rounded-full"></div>}
                  {quarter.isEstimate && <div className="absolute top-1 left-1 w-2 h-2 bg-purple-500 rounded-full"></div>}
                  
                  <div className="text-center">
                    <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 mb-2">{quarter.label}</div>
                    
                    {/* Price row */}
                    <div className="mb-2">
                      <div className="text-xs font-bold text-blue-700 dark:text-blue-300">
                        {quarter.price ? formatCurrency(quarter.price) : 'N/A'}
                      </div>
                      <div className={`text-xs flex items-center justify-center gap-1 ${quarter.priceGrowth !== null && getGrowthIndicator(quarter.priceGrowth).color}`}>
                        {quarter.priceGrowth !== null && (
                          <>
                            <span>{getGrowthIndicator(quarter.priceGrowth).emoji}</span>
                            <span>{formatPercentage(quarter.priceGrowth)}</span>
                          </>
                        )}
                      </div>
                    </div>

                    <div className="border-t border-slate-300 dark:border-slate-500 my-1"></div>

                    {/* EPS row */}
                    <div>
                      <div className="text-xs font-bold text-green-700 dark:text-green-300">
                        {quarter.eps ? formatEPS(quarter.eps) : 'N/A'}
                      </div>
                      <div className={`text-xs flex items-center justify-center gap-1 ${quarter.epsGrowth !== null && getGrowthIndicator(quarter.epsGrowth).color}`}>
                        {quarter.epsGrowth !== null ? (
                          <>
                            <span>{getGrowthIndicator(quarter.epsGrowth).emoji}</span>
                            <span>{formatPercentage(quarter.epsGrowth)}</span>
                          </>
                        ) : quarter.label === 'Q-2' ? (
                          <span className="text-slate-500 dark:text-slate-400">📊</span>
                        ) : quarter.isEstimate ? (
                          <span className="text-purple-600 dark:text-purple-400">🔮</span>
                        ) : (
                          <span className="text-slate-400 dark:text-slate-500">—</span>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Trend Summary */}
          <div className="flex items-center justify-center gap-4 mt-3 text-xs">
            <span className="flex items-center gap-1">
              <span>📈</span>
              <span className="font-medium">Price Recovery</span>
            </span>
            <span className="text-slate-400">•</span>
            <span className="flex items-center gap-1">
              <span>🎢</span>
              <span className="font-medium">EPS Volatility</span>
            </span>
          </div>
        </div>

        {/* Bottom Summary Bar */}
        <div className="bg-gradient-to-r from-slate-100 to-slate-200 dark:from-slate-700 dark:to-slate-600 p-3 rounded-xl text-center">
          <div className="flex items-center justify-center gap-6 text-xs font-medium">
            <span className="flex items-center gap-1">
              <span className="text-slate-600 dark:text-slate-300">📊 QoQ</span>
              <span className={getGrowthIndicator(cardData.eps_quarterly?.qoq_growth_current || 0).color}>
                {getGrowthIndicator(cardData.eps_quarterly?.qoq_growth_current || 0).emoji}
                {formatPercentage(cardData.eps_quarterly?.qoq_growth_current || 0)}
              </span>
            </span>
            <span className="flex items-center gap-1">
              <span className="text-slate-600 dark:text-slate-300">📈 YoY</span>
              <span className={getGrowthIndicator(cardData.eps_quarterly?.yoy_growth_current || 0).color}>
                {getGrowthIndicator(cardData.eps_quarterly?.yoy_growth_current || 0).emoji}
                {formatPercentage(cardData.eps_quarterly?.yoy_growth_current || 0)}
              </span>
            </span>
            <span className="flex items-center gap-1">
              <span className="text-slate-600 dark:text-slate-300">📊 Trend</span>
              <span className="text-slate-700 dark:text-slate-200">
                {cardData.eps_quarterly?.trend_direction === 'UP' ? '📈UP' : cardData.eps_quarterly?.trend_direction === 'DOWN' ? '📉DOWN' : '➡️FLAT'}
              </span>
            </span>
            <span className="flex items-center gap-1">
              <span className="text-slate-600 dark:text-slate-300">⚠️</span>
              <span className={
                cardData.eps_quarterly?.consistency_score === 'HIGH' ? 'text-green-600 dark:text-green-400' :
                cardData.eps_quarterly?.consistency_score === 'MEDIUM' ? 'text-yellow-600 dark:text-yellow-400' :
                'text-red-600 dark:text-red-400'
              }>
                {cardData.eps_quarterly?.consistency_score || 'LOW'}
              </span>
            </span>
          </div>
        </div>

      </div>
    </div>
  )
}