import { getCurrentUser } from '@/lib/server-actions';
import { redirect } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { 
  TrendingUp, 
  TrendingDown, 
  DollarSign, 
  BarChart3, 
  PieChart, 
  Activity,
  Zap,
  Target,
  Clock,
  Eye
} from 'lucide-react';

export const dynamic = 'force-dynamic';

export default async function TradingPage() {
  // Server-side auth check with automatic redirect if not authenticated
  let user = null;
  try {
    const result = await getCurrentUser({});
    user = result?.success ? result.data : null;
  } catch (error) {
    console.error('TradingPage: Failed to get user:', error);
  }

  if (!user) {
    const { redirectToBackendLogin } = await import('@/lib/server/auth');
    redirectToBackendLogin('/trading');
  }

  // Mock data for demonstration
  const portfolioValue = 125430.50;
  const dailyChange = 2.34;
  const dailyChangePercent = 1.85;
  const watchlistCount = 12;
  const activePositions = 8;

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-blue-900/20 dark:to-gray-800">
      {/* Mobile-First Container */}
      <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-6 sm:py-8">
        
        {/* Header Section - Responsive */}
        <div className="mb-6 sm:mb-8">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h1 className="text-2xl sm:text-3xl lg:text-4xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
                📊 Trading Dashboard
              </h1>
              <p className="text-sm sm:text-base text-muted-foreground mt-1">
                Welcome back, <span className="font-semibold">{user.email}</span>
              </p>
            </div>
            
            {/* Quick Actions - Mobile Optimized */}
            <div className="flex flex-wrap gap-2 sm:gap-3">
              <Button size="sm" className="bg-green-600 hover:bg-green-700 text-white">
                <TrendingUp className="h-4 w-4 mr-2" />
                <span className="hidden sm:inline">Quick Buy</span>
                <span className="sm:hidden">Buy</span>
              </Button>
              <Button size="sm" variant="outline" className="border-red-200 hover:bg-red-50 dark:border-red-800 dark:hover:bg-red-900/20">
                <TrendingDown className="h-4 w-4 mr-2" />
                <span className="hidden sm:inline">Quick Sell</span>
                <span className="sm:hidden">Sell</span>
              </Button>
            </div>
          </div>
        </div>

        {/* Portfolio Summary Cards - Responsive Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6 mb-6 sm:mb-8">
          
          {/* Portfolio Value Card */}
          <Card className="relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-blue-500/10 to-indigo-500/10" />
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
                <DollarSign className="h-4 w-4" />
                Portfolio Value
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="text-xl sm:text-2xl font-bold text-blue-600">
                ${portfolioValue.toLocaleString()}
              </div>
              <div className="flex items-center gap-1 mt-1">
                <TrendingUp className="h-3 w-3 text-green-600" />
                <span className="text-xs sm:text-sm text-green-600 font-medium">
                  +${dailyChange} ({dailyChangePercent}%)
                </span>
              </div>
            </CardContent>
          </Card>

          {/* Active Positions Card */}
          <Card className="relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-green-500/10 to-emerald-500/10" />
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
                <Target className="h-4 w-4" />
                Active Positions
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="text-xl sm:text-2xl font-bold text-green-600">
                {activePositions}
              </div>
              <Badge variant="secondary" className="mt-2 text-xs">
                <Activity className="h-3 w-3 mr-1" />
                All performing
              </Badge>
            </CardContent>
          </Card>

          {/* Watchlist Card */}
          <Card className="relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-orange-500/10 to-yellow-500/10" />
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
                <Eye className="h-4 w-4" />
                Watchlist
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="text-xl sm:text-2xl font-bold text-orange-600">
                {watchlistCount}
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                Stocks tracked
              </p>
            </CardContent>
          </Card>

          {/* Market Status Card */}
          <Card className="relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-purple-500/10 to-pink-500/10" />
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
                <Clock className="h-4 w-4" />
                Market Status
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                <span className="text-sm sm:text-base font-semibold text-green-600">Open</span>
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                Closes in 4h 23m
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Main Content Grid - Responsive Layout */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 sm:gap-8">
          
          {/* Left Column - Charts & Analysis */}
          <div className="lg:col-span-2 space-y-6">
            
            {/* Portfolio Chart Card */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <BarChart3 className="h-5 w-5" />
                  Portfolio Performance
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-48 sm:h-64 lg:h-80 bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-lg flex items-center justify-center">
                  <div className="text-center">
                    <BarChart3 className="h-12 w-12 text-blue-500 mx-auto mb-4" />
                    <p className="text-muted-foreground">Interactive chart coming soon</p>
                    <p className="text-xs text-muted-foreground mt-1">Real-time portfolio tracking</p>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Recent Trades */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  Recent Activity
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {[1, 2, 3].map((i) => (
                    <div key={i} className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-indigo-500 rounded-full flex items-center justify-center text-white text-xs font-bold">
                          {i === 1 ? 'AA' : i === 2 ? 'GO' : 'MS'}
                        </div>
                        <div>
                          <p className="font-medium text-sm">
                            {i === 1 ? 'AAPL' : i === 2 ? 'GOOGL' : 'MSFT'}
                          </p>
                          <p className="text-xs text-muted-foreground">
                            {i === 1 ? 'Bought 10 shares' : i === 2 ? 'Sold 5 shares' : 'Bought 15 shares'}
                          </p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className={`text-sm font-medium ${i === 2 ? 'text-red-600' : 'text-green-600'}`}>
                          {i === 2 ? '-$1,250' : i === 1 ? '+$1,750' : '+$4,200'}
                        </p>
                        <p className="text-xs text-muted-foreground">2h ago</p>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Right Column - Watchlist & Actions */}
          <div className="space-y-6">
            
            {/* Quick Stats */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Zap className="h-5 w-5" />
                  Quick Stats
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Total Return</span>
                  <span className="font-semibold text-green-600">+12.8%</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Best Performer</span>
                  <span className="font-semibold">AAPL +5.2%</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Cash Available</span>
                  <span className="font-semibold">$25,430</span>
                </div>
              </CardContent>
            </Card>

            {/* Top Watchlist */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <PieChart className="h-5 w-5" />
                  Top Watchlist
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {['AAPL', 'GOOGL', 'MSFT', 'TSLA'].map((symbol, i) => (
                  <div key={symbol} className="flex items-center justify-between p-2 hover:bg-muted/50 rounded-lg transition-colors">
                    <div className="flex items-center gap-2">
                      <div className="w-6 h-6 bg-gradient-to-br from-blue-500 to-indigo-500 rounded text-white text-xs font-bold flex items-center justify-center">
                        {symbol[0]}
                      </div>
                      <span className="font-medium text-sm">{symbol}</span>
                    </div>
                    <div className="text-right">
                      <p className="text-sm font-medium">${(150 + i * 25).toFixed(2)}</p>
                      <p className={`text-xs ${i % 2 === 0 ? 'text-green-600' : 'text-red-600'}`}>
                        {i % 2 === 0 ? '+2.1%' : '-1.3%'}
                      </p>
                    </div>
                  </div>
                ))}
              </CardContent>
            </Card>

            {/* Coming Soon Card */}
            <Card className="border-dashed border-2 border-muted-foreground/20">
              <CardContent className="pt-6">
                <div className="text-center">
                  <div className="w-12 h-12 bg-gradient-to-br from-orange-500 to-yellow-500 rounded-full flex items-center justify-center text-white text-xl mb-4 mx-auto">
                    🚀
                  </div>
                  <h3 className="font-semibold mb-2">Advanced Features</h3>
                  <p className="text-sm text-muted-foreground mb-4">
                    Real-time trading, advanced charts, and AI-powered insights coming soon!
                  </p>
                  <Button variant="outline" size="sm" className="w-full">
                    Join Waitlist
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
}
