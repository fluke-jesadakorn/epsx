import { PositionAction, type PortfolioPosition, type PortfolioData } from '@/types/analytics';

export const mockPortfolioPositions: PortfolioPosition[] = [
  {
    symbol: 'MSFT',
    rank: 1,
    actionStatus: PositionAction.KEEP,
    actionPhase: {
      start: 'Jul 30, 2025',
      end: '2025-10-28'
    },
    daysRemaining: 100,
    performance: 5.49,
    quarters: [
      {
        date: 'Apr 30, 2025',
        growth: 7.12,
        eps: 3.46,
        price: -0.34
      },
      {
        date: 'Jul 30, 2025', 
        growth: 5.49,
        eps: 3.65,
        price: 4.94
      }
    ],
    nextAnnouncement: '2025-10-28',
    gradientClass: 'bg-gradient-to-br from-pink-500 via-purple-600 to-purple-700'
  },
  {
    symbol: 'AMZN',
    rank: 2,
    actionStatus: PositionAction.KEEP,
    actionPhase: {
      start: 'Jul 31, 2025',
      end: '2025-10-23'
    },
    daysRemaining: 100,
    performance: 5.66,
    quarters: [
      {
        date: 'May 1, 2025',
        growth: -14.52,
        eps: 1.59,
        price: 3.86
      },
      {
        date: 'Jul 31, 2025',
        growth: 5.66,
        eps: 1.68,
        price: 2.69
      }
    ],
    nextAnnouncement: '2025-10-23',
    gradientClass: 'bg-gradient-to-br from-purple-600 via-blue-600 to-blue-700'
  },
  {
    symbol: 'META',
    rank: 3,
    actionStatus: PositionAction.KEEP,
    actionPhase: {
      start: 'Jul 30, 2025',
      end: '2025-10-22'
    },
    daysRemaining: 100,
    performance: 11.04,
    quarters: [
      {
        date: 'Apr 30, 2025',
        growth: -19.83,
        eps: 6.43,
        price: -0.15
      },
      {
        date: 'Jul 30, 2025',
        growth: 11.04,
        eps: 7.14,
        price: 8.74
      }
    ],
    nextAnnouncement: '2025-10-22',
    gradientClass: 'bg-gradient-to-br from-blue-600 via-cyan-600 to-teal-600'
  },
  {
    symbol: 'TSLA',
    rank: 4,
    actionStatus: PositionAction.KEEP,
    actionPhase: {
      start: 'Jul 23, 2025',
      end: '2025-10-15'
    },
    daysRemaining: 100,
    performance: 48.15,
    quarters: [
      {
        date: 'Apr 22, 2025',
        growth: -63.01,
        eps: 0.27,
        price: 20.86
      },
      {
        date: 'Jul 23, 2025',
        growth: 48.15,
        eps: 0.40,
        price: 39.73
      }
    ],
    nextAnnouncement: '2025-10-15',
    gradientClass: 'bg-gradient-to-br from-cyan-500 via-blue-500 to-purple-600'
  }
];

export const mockPortfolioData: PortfolioData = {
  positions: mockPortfolioPositions,
  processingTime: 66891,
  lastUpdated: new Date().toISOString()
};