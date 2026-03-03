import { Button } from '@/components/ui/button';
import { LineChart, TrendingUp, Users, Zap } from 'lucide-react';
import Link from 'next/link';
import type { CSSProperties } from 'react';
import { ShareButton } from './share-button';

interface HeroSectionProps {
  style?: CSSProperties;
  className?: string;
}

const STATS = [
  { number: '24/7', label: '🔄 Latest Updates', Icon: Zap, gradient: 'from-blue-500 to-cyan-500' },
  { number: '100+', label: '📊 Stock Analytics', Icon: TrendingUp, gradient: 'from-yellow-500 to-orange-500' },
  { number: '< 1s', label: '⚡ Response Time', Icon: Users, gradient: 'from-green-500 to-emerald-500' },
];

const HeroSection: React.FC<HeroSectionProps> = ({ style, className }) => {
  return (
    <div
      className={`relative w-full min-h-[85vh] flex items-center justify-center overflow-hidden ${className ?? ''}`}
      style={style}
    >
      <div className="relative text-center space-y-12 max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-16 sm:py-20 z-[1]">
        <div className="space-y-8">
          <div className="space-y-6">
            <div className="inline-block animate-slide-up">
              <div className="mb-4 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-gradient-to-r from-primary/10 to-secondary/10 border border-primary/20 backdrop-blur-sm">
                <TrendingUp className="h-4 w-4 text-primary" />
                <span className="text-sm font-medium text-primary">
                  Performance Analytics Platform
                </span>
              </div>
              <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl xl:text-8xl font-bold leading-tight">
                <span className="block">📈 Track Your</span>
                <span className="block bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-transparent animate-gradient-x">
                  Performance Growth
                </span>
                <span className="block mt-2">Metrics ✨</span>
              </h1>
            </div>

            <div className="animate-slide-up-delayed">
              <p className="text-lg sm:text-xl md:text-2xl text-gray-600 dark:text-gray-300 max-w-4xl mx-auto leading-relaxed">
                🚀 Discover comprehensive data insights with our advanced analytics platform!
                <span className="block mt-2 font-bold">
                  <span className="bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
                    Make informed decisions with real-time insights
                  </span>
                  <span className="ml-2">📈</span>
                </span>
              </p>
            </div>
          </div>

          <div className="flex flex-col sm:flex-row gap-4 sm:gap-6 justify-center items-center animate-slide-up-delayed-2">
            <Link href="/analytics" className="w-full sm:w-auto">
              <Button className="w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-2xl shadow-2xl hover:shadow-orange-300/50 hover:scale-105 transition-all duration-300 group">
                <LineChart className="mr-3 h-6 w-6 group-hover:animate-bounce-gentle" />
                🚀 Start Exploration
              </Button>
            </Link>
            <ShareButton />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 sm:gap-8 mt-16 animate-fade-in-delayed-3">
            {STATS.map((stat, index) => (
              <div
                key={stat.label}
                className="relative bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl rounded-2xl p-8 shadow-2xl border border-orange-200/50 dark:border-orange-400/20 hover:scale-105 transition-all duration-300 group overflow-hidden"
                style={{ animationDelay: `${index * 0.1}s` }}
              >
                <div className={`absolute inset-0 bg-gradient-to-br ${stat.gradient} opacity-5 group-hover:opacity-10 transition-opacity duration-300`} />
                <div className="relative z-10 text-center">
                  <stat.Icon className="h-10 w-10 mx-auto mb-4 text-orange-500 group-hover:animate-bounce-gentle transition-colors duration-300" />
                  <div className={`text-3xl sm:text-4xl font-bold bg-gradient-to-r ${stat.gradient} bg-clip-text text-transparent mb-2`}>
                    {stat.number}
                  </div>
                  <div className="text-sm font-medium text-gray-600 dark:text-gray-300">
                    {stat.label}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default HeroSection;
