'use client';

import { ReactNode, useRef, useState, useCallback } from 'react';
import { useTouchGestures } from '@/hooks/useTouchGestures';
import { Card } from '@epsx/ui';

interface SwipeAction {
  icon: ReactNode;
  label: string;
  color: string;
  onAction: () => void;
}

interface SwipeableCardProps {
  children: ReactNode;
  leftAction?: SwipeAction;
  rightAction?: SwipeAction;
  onSwipeLeft?: () => void;
  onSwipeRight?: () => void;
  swipeThreshold?: number;
  className?: string;
  disabled?: boolean;
}

export function SwipeableCard({
  children,
  leftAction,
  rightAction,
  onSwipeLeft,
  onSwipeRight,
  swipeThreshold = 100,
  className = '',
  disabled = false
}: SwipeableCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const [isRevealed, setIsRevealed] = useState<'left' | 'right' | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  const handleSwipeLeft = useCallback(() => {
    if (disabled) return;
    
    if (rightAction) {
      setIsRevealed('right');
    }
    onSwipeLeft?.();
  }, [disabled, rightAction, onSwipeLeft]);

  const handleSwipeRight = useCallback(() => {
    if (disabled) return;
    
    if (leftAction) {
      setIsRevealed('left');
    }
    onSwipeRight?.();
  }, [disabled, leftAction, onSwipeRight]);

  const handlePanStart = useCallback(() => {
    setIsDragging(true);
  }, []);

  const handlePanEnd = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleTap = useCallback(() => {
    if (isRevealed) {
      setIsRevealed(null);
    }
  }, [isRevealed]);

  const { bind, transform } = useTouchGestures({
    onSwipeLeft: handleSwipeLeft,
    onSwipeRight: handleSwipeRight,
    onPanStart: handlePanStart,
    onPanEnd: handlePanEnd,
    onTap: handleTap,
    swipeThreshold,
    enabled: !disabled
  });

  const executeAction = (action: SwipeAction) => {
    action.onAction();
    setIsRevealed(null);
  };

  return (
    <div className="relative overflow-hidden">
      {/* Background Actions */}
      {leftAction && (
        <div className={`
          absolute inset-y-0 left-0 flex items-center justify-start pl-4 pr-6
          transition-all duration-200 z-0
          ${isRevealed === 'left' ? 'w-24 opacity-100' : 'w-0 opacity-0'}
        `}>
          <button
            onClick={() => executeAction(leftAction)}
            className={`
              flex flex-col items-center gap-1 p-2 rounded-lg min-w-16
              transition-all duration-200 hover:scale-105 active:scale-95
              ${leftAction.color}
            `}
          >
            {leftAction.icon}
            <span className="text-xs font-medium text-white">
              {leftAction.label}
            </span>
          </button>
        </div>
      )}

      {rightAction && (
        <div className={`
          absolute inset-y-0 right-0 flex items-center justify-end pr-4 pl-6
          transition-all duration-200 z-0
          ${isRevealed === 'right' ? 'w-24 opacity-100' : 'w-0 opacity-0'}
        `}>
          <button
            onClick={() => executeAction(rightAction)}
            className={`
              flex flex-col items-center gap-1 p-2 rounded-lg min-w-16
              transition-all duration-200 hover:scale-105 active:scale-95
              ${rightAction.color}
            `}
          >
            {rightAction.icon}
            <span className="text-xs font-medium text-white">
              {rightAction.label}
            </span>
          </button>
        </div>
      )}

      {/* Main Card */}
      <Card
        ref={cardRef}
        {...bind()}
        className={`
          relative z-10 cursor-pointer transition-all duration-200
          ${isDragging ? 'shadow-lg scale-[0.98]' : 'shadow-sm'}
          ${disabled ? 'opacity-60 cursor-not-allowed' : ''}
          ${className}
        `}
        style={{
          transform: isRevealed ? `translateX(${isRevealed === 'left' ? '80px' : '-80px'})` : transform
        }}
      >
        {children}
      </Card>

      {/* Overlay for closing revealed actions */}
      {isRevealed && (
        <div
          className="fixed inset-0 z-5 bg-transparent"
          onClick={() => setIsRevealed(null)}
        />
      )}
    </div>
  );
}