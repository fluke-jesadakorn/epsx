'use client';

import { useState } from 'react';
import { PositionAction, type PositionCardProps } from '@/types/analytics';

export default function PositionCard({ position, onActionChange }: PositionCardProps) {
  const [isToggling, setIsToggling] = useState(false);

  const handleActionToggle = () => {
    if (isToggling) return;
    
    setIsToggling(true);
    const newAction = position.actionStatus === PositionAction.KEEP 
      ? PositionAction.STOP 
      : PositionAction.KEEP;
    
    onActionChange(position.symbol, newAction);
    
    setTimeout(() => setIsToggling(false), 300);
  };

  const getActionStatusConfig = () => {
    switch (position.actionStatus) {
      case PositionAction.KEEP:
        return {
          text: 'KEEP',
          bgClass: 'bg-green-500/20 border-green-500/50',
          textClass: 'text-green-400',
          dotClass: 'bg-green-400'
        };
      case PositionAction.STOP:
        return {
          text: 'STOP', 
          bgClass: 'bg-red-500/20 border-red-500/50',
          textClass: 'text-red-400',
          dotClass: 'bg-red-400'
        };
      case PositionAction.TRACK:
        return {
          text: 'TRACK',
          bgClass: 'bg-blue-500/20 border-blue-500/50', 
          textClass: 'text-blue-400',
          dotClass: 'bg-blue-400'
        };
    }
  };

  const statusConfig = getActionStatusConfig();

  return (
    <div className={`
      relative rounded-3xl border border-white/10 backdrop-blur-xl shadow-2xl 
      transition-all duration-300 hover:scale-[1.02] hover:shadow-3xl
      ${position.gradientClass} p-6 h-full
    `}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="bg-gradient-to-br from-pink-400 to-purple-600 rounded-2xl px-3 py-1 text-sm font-bold text-white">
            {position.symbol} #{position.rank}
          </div>
          <button className="p-1.5 rounded-lg bg-white/10 hover:bg-white/20 transition-colors">
            <svg className="h-4 w-4 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
          </button>
        </div>
      </div>

      {/* Action Status */}
      <button 
        onClick={handleActionToggle}
        disabled={isToggling}
        className={`
          w-full mb-6 px-4 py-3 rounded-2xl border transition-all duration-300
          ${statusConfig.bgClass} ${statusConfig.textClass}
          hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed
          flex items-center justify-center gap-2 font-bold text-sm
        `}
      >
        <div className={`w-3 h-3 rounded-full ${statusConfig.dotClass} animate-pulse`} />
        {statusConfig.text}
      </button>

      {/* Action Phase */}
      <div className="mb-6">
        <div className="text-cyan-400 text-sm font-semibold mb-2">Action Phase</div>
        <div className="flex justify-between text-xs text-gray-300">
          <span>{position.actionPhase.start}</span>
          <span>{position.actionPhase.end}</span>
        </div>
        
        {/* Progress Bar */}
        <div className="mt-3 bg-gray-700/50 rounded-full h-2 overflow-hidden">
          <div 
            className="bg-gradient-to-r from-cyan-400 to-blue-500 h-full rounded-full transition-all duration-1000"
            style={{ width: '100%' }}
          />
        </div>
        <div className="text-right mt-1">
          <span className="text-2xl font-bold text-cyan-400">{position.daysRemaining}</span>
          <span className="text-xs text-gray-400 ml-1">days</span>
        </div>
      </div>

      {/* Performance Indicator */}
      <div className="mb-6 flex justify-center">
        <div className="bg-green-500/20 rounded-2xl px-4 py-2 flex items-center gap-2">
          <svg className="h-5 w-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
          </svg>
          <span className="text-green-400 font-bold text-lg">
            +{position.performance.toFixed(2)}%
          </span>
        </div>
      </div>

      {/* Quarterly Data */}
      <div className="space-y-4 mb-6">
        {position.quarters.map((quarter, idx) => (
          <div key={idx} className="bg-white/5 rounded-2xl p-4 border border-white/10">
            <div className="text-cyan-400 text-sm font-medium mb-2">{quarter.date}</div>
            <div className="text-xs space-y-1 text-gray-300">
              <div>Growth: {quarter.growth > 0 ? '+' : ''}{quarter.growth.toFixed(2)}% | EPS: {quarter.eps.toFixed(2)}</div>
              <div>Price: {quarter.price > 0 ? '+' : ''}{quarter.price.toFixed(2)}% | {new Intl.NumberFormat('en-US', {
                style: 'currency',
                currency: position.currency || 'USD',
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              }).format(Math.random() * 400 + 200)}</div>
            </div>
          </div>
        ))}
      </div>

      {/* Next Announcement */}
      <div className="bg-white/5 rounded-2xl p-3 border border-white/10">
        <div className="text-cyan-400 text-sm font-medium">Next: {position.nextAnnouncement}</div>
      </div>
    </div>
  );
}