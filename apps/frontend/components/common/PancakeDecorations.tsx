'use client';

import React from 'react';

interface PancakeDecorationsProps {
  className?: string;
  variant?: 'full' | 'subtle' | 'minimal';
}

export const PancakeDecorations: React.FC<PancakeDecorationsProps> = ({
  className = '',
  variant = 'full',
}) => {
  const isSubtle = variant === 'subtle';
  const isMinimal = variant === 'minimal';

  if (isMinimal) {
    return (
      <div className={`fixed inset-0 pointer-events-none z-0 ${className}`}>
        {/* Just a few subtle elements */}
        <div className="absolute top-1/4 left-1/4 w-24 h-24 bg-gradient-to-br from-orange-400/3 to-yellow-400/3 rounded-full blur-2xl animate-float" />
        <div className="absolute bottom-1/3 right-1/4 w-28 h-28 bg-gradient-to-br from-blue-400/3 to-cyan-400/3 rounded-full blur-2xl animate-bounce-gentle" />
      </div>
    );
  }

  return (
    <div
      className={`fixed inset-0 pointer-events-none z-0 overflow-hidden ${className}`}
    >
      {/* Enhanced floating pancake emojis with better positioning */}
      <div
        className="pancake-emoji text-4xl opacity-15 animate-float-gentle"
        style={{ top: '8%', left: '3%', animationDelay: '0s' }}
      >
        🥞
      </div>
      <div
        className="pancake-emoji text-3xl opacity-10 animate-bounce-gentle"
        style={{ top: '15%', right: '8%', animationDelay: '1s' }}
      >
        🧁
      </div>
      <div
        className="pancake-emoji text-5xl opacity-12 animate-float"
        style={{ bottom: '25%', left: '6%', animationDelay: '2s' }}
      >
        🍰
      </div>
      <div
        className="pancake-emoji text-4xl opacity-15 animate-pulse-gentle"
        style={{ bottom: '12%', right: '12%', animationDelay: '3s' }}
      >
        🥞
      </div>
      <div
        className="pancake-emoji text-3xl opacity-8 animate-bounce-gentle"
        style={{ top: '45%', left: '2%', animationDelay: '4s' }}
      >
        �
      </div>
      <div
        className="pancake-emoji text-4xl opacity-12 animate-float-gentle"
        style={{ top: '65%', right: '4%', animationDelay: '5s' }}
      >
        🍰
      </div>
      <div
        className="pancake-emoji text-2xl opacity-8 animate-float"
        style={{ top: '30%', left: '15%', animationDelay: '6s' }}
      >
        🥯
      </div>
      <div
        className="pancake-emoji text-3xl opacity-10 animate-bounce-gentle"
        style={{ bottom: '40%', right: '20%', animationDelay: '7s' }}
      >
        🍯
      </div>
      <div
        className="pancake-emoji text-4xl opacity-12 animate-pulse-gentle"
        style={{ top: '55%', right: '25%', animationDelay: '8s' }}
      >
        �
      </div>

      {/* Additional food emojis for more PancakeSwap vibes */}
      <div
        className="pancake-emoji text-2xl opacity-6 animate-float-gentle"
        style={{ top: '75%', left: '25%', animationDelay: '9s' }}
      >
        🍓
      </div>
      <div
        className="pancake-emoji text-3xl opacity-8 animate-bounce-gentle"
        style={{ top: '25%', right: '30%', animationDelay: '10s' }}
      >
        �
      </div>
      <div
        className="pancake-emoji text-2xl opacity-7 animate-float"
        style={{ bottom: '60%', left: '30%', animationDelay: '11s' }}
      >
        🥛
      </div>

      {/* Enhanced gradient orbs with more vibrant colors */}
      <div className="absolute -top-20 -left-20 w-48 h-48 bg-gradient-to-br from-orange-400/8 to-yellow-400/8 rounded-full blur-3xl animate-float" />
      <div className="absolute top-1/4 left-1/3 w-36 h-36 bg-gradient-to-br from-orange-400/6 to-yellow-400/6 rounded-full blur-2xl animate-bounce-gentle" />
      <div className="absolute bottom-1/3 right-1/4 w-52 h-52 bg-gradient-to-br from-blue-400/7 to-cyan-400/7 rounded-full blur-3xl animate-pulse-gentle" />
      <div className="absolute top-1/2 right-1/3 w-32 h-32 bg-gradient-to-br from-purple-400/6 to-pink-400/6 rounded-full blur-2xl animate-float-gentle" />
      <div className="absolute -bottom-16 -right-16 w-44 h-44 bg-gradient-to-br from-green-400/5 to-emerald-400/5 rounded-full blur-3xl animate-bounce-gentle" />
      <div className="absolute top-2/3 left-1/5 w-28 h-28 bg-gradient-to-br from-rose-400/5 to-pink-400/5 rounded-full blur-xl animate-float" />

      {/* Geometric shapes for extra visual interest */}
      {!isSubtle && (
        <>
          <div className="absolute top-1/6 left-1/2 w-16 h-16 bg-gradient-to-br from-orange-300/8 to-yellow-300/8 rounded-2xl rotate-45 animate-spin-slow" />
          <div className="absolute bottom-1/4 left-1/3 w-12 h-12 bg-gradient-to-br from-blue-300/6 to-cyan-300/6 rounded-full animate-bounce-gentle" />
          <div className="absolute top-3/4 right-1/2 w-20 h-20 bg-gradient-to-br from-purple-300/5 to-pink-300/5 rounded-3xl rotate-12 animate-float-gentle" />
          <div className="absolute top-1/3 right-1/5 w-14 h-14 bg-gradient-to-br from-green-300/6 to-emerald-300/6 rounded-xl -rotate-12 animate-pulse-gentle" />

          {/* Additional floating elements */}
          <div className="absolute top-1/8 right-1/4 w-8 h-8 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full animate-bounce-gentle" />
          <div className="absolute bottom-1/8 left-1/4 w-6 h-6 bg-gradient-to-br from-blue-400/8 to-cyan-400/8 rounded-full animate-float" />
          <div className="absolute top-5/6 right-1/6 w-10 h-10 bg-gradient-to-br from-purple-400/7 to-pink-400/7 rounded-full animate-pulse-gentle" />
        </>
      )}

      {/* Sparkle effects for extra PancakeSwap magic */}
      {!isSubtle && (
        <>
          <div
            className="absolute top-1/5 left-1/6 w-2 h-2 bg-yellow-400/30 rounded-full animate-ping"
            style={{ animationDelay: '0s' }}
          />
          <div
            className="absolute top-2/5 right-1/8 w-1 h-1 bg-orange-400/40 rounded-full animate-ping"
            style={{ animationDelay: '1s' }}
          />
          <div
            className="absolute bottom-1/5 left-1/3 w-2 h-2 bg-blue-400/25 rounded-full animate-ping"
            style={{ animationDelay: '2s' }}
          />
          <div
            className="absolute top-3/5 right-1/3 w-1 h-1 bg-purple-400/35 rounded-full animate-ping"
            style={{ animationDelay: '3s' }}
          />
          <div
            className="absolute bottom-2/5 right-1/6 w-2 h-2 bg-green-400/20 rounded-full animate-ping"
            style={{ animationDelay: '4s' }}
          />
        </>
      )}

      {/* Mesh gradient overlays for depth */}
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_20%_30%,_rgba(255,133,27,0.04)_0%,_transparent_50%)] animate-pulse-slow" />
      <div
        className="absolute inset-0 bg-[radial-gradient(circle_at_80%_70%,_rgba(59,130,246,0.03)_0%,_transparent_50%)] animate-pulse-slow"
        style={{ animationDelay: '2s' }}
      />
      <div
        className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.02)_0%,_transparent_60%)] animate-pulse-slow"
        style={{ animationDelay: '4s' }}
      />
    </div>
  );
};

export default PancakeDecorations;
