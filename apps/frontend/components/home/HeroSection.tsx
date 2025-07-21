'use client';

import { LineChart, Share2, TrendingUp, Users, Zap } from 'lucide-react';
import Link from 'next/link';
import React, { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { WithLoading } from '@/components/common/withLoading';
import { Button } from '@/components/ui/button';

import type { CSSProperties } from 'react';

interface HeroSectionProps {
  style?: CSSProperties;
  className?: string;
}

const HeroSection: React.FC<HeroSectionProps> = ({ style, className }) => {
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate loading delay
    const timer = setTimeout(() => {
      setLoading(false);
    }, 1500);
    return () => clearTimeout(timer);
  }, []);

  const handleShare = () => {
    const url = window.location.href;
    navigator.clipboard.writeText(url).then(() => {
      toast.success('URL copied to clipboard!');
    });
  };

  return (
    <div
      className={`relative w-full min-h-[85vh] flex items-center justify-center overflow-hidden ${className || ''}`}
      style={style}
    >
      {/* PancakeSwap-style floating elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-20 left-10 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float" />
        <div className="absolute top-32 right-20 w-12 h-12 bg-gradient-to-br from-amber-400/20 to-orange-400/20 rounded-full animate-bounce-gentle" />
        <div className="absolute bottom-40 left-20 w-8 h-8 bg-gradient-to-br from-yellow-400/20 to-amber-400/20 rounded-full animate-pulse-gentle" />
        <div className="absolute bottom-20 right-10 w-20 h-20 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float-reverse" />

        {/* Decorative pancake-like shapes with enhanced styling */}
        <div className="absolute top-1/4 left-1/4 text-6xl opacity-10 animate-spin-slow">
          🥞
        </div>
        <div className="absolute bottom-1/4 right-1/4 text-4xl opacity-20 animate-bounce-gentle">
          ✨
        </div>
        <div className="absolute top-3/4 left-10 text-3xl opacity-15 animate-float-gentle">
          🚀
        </div>
        <div className="absolute top-1/2 right-1/4 text-5xl opacity-8 animate-wiggle">
          💰
        </div>
        <div className="absolute bottom-1/3 left-1/3 text-2xl opacity-25 animate-pulse">
          📈
        </div>
      </div>

      <div className="relative text-center space-y-12 max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-16 sm:py-20 z-10">
        <WithLoading loading={loading} className="space-y-8">
          {/* Main heading with enhanced PancakeSwap-style typography */}
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
                <span className="block bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent animate-gradient-x">
                  Performance Growth
                </span>
                <span className="block mt-2">Metrics ✨</span>
              </h1>
            </div>

            {/* Enhanced subtitle with PancakeSwap vibes */}
            <div className="animate-slide-up-delayed">
              <p className="text-lg sm:text-xl md:text-2xl text-gray-600 dark:text-gray-300 max-w-4xl mx-auto leading-relaxed">
                🚀 Discover the sweetest data insights with our comprehensive
                analytics platform!
                <span className="block mt-2 bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent font-bold">
                  Make informed decisions with real-time insights 📈
                </span>
              </p>
            </div>
          </div>

          {/* Enhanced CTA buttons with PancakeSwap styling */}
          <div className="flex flex-col sm:flex-row gap-4 sm:gap-6 justify-center items-center animate-slide-up-delayed-2">
            <Link href="/trading" className="w-full sm:w-auto">
              <Button className="w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-2xl shadow-2xl hover:shadow-orange-300/50 hover:scale-105 transition-all duration-300 group">
                <LineChart className="mr-3 h-6 w-6 group-hover:animate-bounce-gentle" />
                🚀 Start Exploration
              </Button>
            </Link>

            <Button
              onClick={handleShare}
              className="w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-white/10 backdrop-blur-sm border-2 border-orange-300/50 text-orange-600 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-900/20 rounded-2xl shadow-xl hover:shadow-orange-300/30 hover:scale-105 transition-all duration-300 group"
            >
              <Share2 className="mr-3 h-6 w-6 group-hover:animate-wiggle" />
              📤 Share Platform
            </Button>
          </div>

          {/* Stats grid with enhanced PancakeSwap styling */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 sm:gap-8 mt-16 animate-fade-in-delayed-3">
            {[
              {
                number: '24/7',
                label: '🔄 Latest Updates',
                icon: Zap,
                gradient: 'from-blue-500 to-cyan-500',
              },
              {
                number: '100+',
                label: '📊 Stock Analytics',
                icon: TrendingUp,
                gradient: 'from-yellow-500 to-orange-500',
              },
              {
                number: '< 1s',
                label: '⚡ Response Time',
                icon: Users,
                gradient: 'from-green-500 to-emerald-500',
              },
            ].map((stat, index) => {
              const IconComponent = stat.icon;
              return (
                <div
                  key={index}
                  className="relative bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl rounded-2xl p-8 shadow-2xl border border-orange-200/50 dark:border-orange-400/20 hover:scale-105 transition-all duration-300 group overflow-hidden"
                  style={{ animationDelay: `${index * 0.1}s` }}
                >
                  {/* Card background decoration */}
                  <div
                    className={`absolute inset-0 bg-gradient-to-br ${stat.gradient} opacity-5 group-hover:opacity-10 transition-opacity duration-300`}
                  />
                  <div className="relative z-10 text-center">
                    <IconComponent className="h-10 w-10 mx-auto mb-4 text-orange-500 group-hover:animate-bounce-gentle transition-colors duration-300" />
                    <div
                      className={`text-3xl sm:text-4xl font-bold bg-gradient-to-r ${stat.gradient} bg-clip-text text-transparent mb-2`}
                    >
                      {stat.number}
                    </div>
                    <div className="text-sm font-medium text-gray-600 dark:text-gray-300">
                      {stat.label}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </WithLoading>
      </div>
    </div>
  );
};

export default HeroSection;
