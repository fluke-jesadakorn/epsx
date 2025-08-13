'use client';

import { ReactNode, useRef, useState, useCallback } from 'react';
import { Button } from '@epsx/ui';

interface LongPressButtonProps {
  children: ReactNode;
  onLongPress: () => void;
  onPress?: () => void;
  longPressDuration?: number;
  hapticFeedback?: boolean;
  showProgress?: boolean;
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  className?: string;
  disabled?: boolean;
}

export function LongPressButton({
  children,
  onLongPress,
  onPress,
  longPressDuration = 800,
  hapticFeedback = true,
  showProgress = true,
  variant = 'default',
  size = 'default',
  className = '',
  disabled = false
}: LongPressButtonProps) {
  const timerRef = useRef<NodeJS.Timeout>();
  const progressRef = useRef<HTMLDivElement>(null);
  const [isPressed, setIsPressed] = useState(false);
  const [progress, setProgress] = useState(0);

  const triggerHaptic = useCallback(() => {
    if (hapticFeedback && 'vibrate' in navigator) {
      navigator.vibrate(50);
    }
  }, [hapticFeedback]);

  const startLongPress = useCallback(() => {
    if (disabled) return;
    
    setIsPressed(true);
    setProgress(0);
    
    const startTime = Date.now();
    const interval = 16; // ~60fps
    
    const updateProgress = () => {
      const elapsed = Date.now() - startTime;
      const newProgress = Math.min((elapsed / longPressDuration) * 100, 100);
      setProgress(newProgress);
      
      if (elapsed >= longPressDuration) {
        // Long press completed
        triggerHaptic();
        onLongPress();
        endLongPress();
      } else {
        timerRef.current = setTimeout(updateProgress, interval);
      }
    };
    
    timerRef.current = setTimeout(updateProgress, interval);
  }, [disabled, longPressDuration, onLongPress, triggerHaptic]);

  const endLongPress = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }
    
    setIsPressed(false);
    setProgress(0);
  }, []);

  const handleClick = useCallback(() => {
    if (disabled) return;
    
    if (progress < 100) {
      // Short press - call onPress if provided
      onPress?.();
    }
    
    endLongPress();
  }, [disabled, progress, onPress, endLongPress]);

  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    e.preventDefault();
    startLongPress();
  }, [startLongPress]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    startLongPress();
  }, [startLongPress]);

  const handleTouchEnd = useCallback((e: React.TouchEvent) => {
    e.preventDefault();
    handleClick();
  }, [handleClick]);

  const handleMouseUp = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    handleClick();
  }, [handleClick]);

  const handleTouchCancel = useCallback(() => {
    endLongPress();
  }, [endLongPress]);

  const handleMouseLeave = useCallback(() => {
    endLongPress();
  }, [endLongPress]);

  return (
    <Button
      variant={variant}
      size={size}
      className={`
        relative overflow-hidden transition-all duration-150
        ${isPressed ? 'scale-95' : 'scale-100'}
        ${className}
      `}
      disabled={disabled}
      onTouchStart={handleTouchStart}
      onTouchEnd={handleTouchEnd}
      onTouchCancel={handleTouchCancel}
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseLeave}
      style={{
        userSelect: 'none',
        WebkitUserSelect: 'none',
        WebkitTouchCallout: 'none'
      }}
    >
      {children}
      
      {/* Progress indicator */}
      {showProgress && isPressed && (
        <div
          ref={progressRef}
          className="absolute inset-0 bg-white/20 transition-all duration-75 ease-linear"
          style={{
            transform: `scaleX(${progress / 100})`,
            transformOrigin: 'left center'
          }}
        />
      )}
      
      {/* Ripple effect */}
      {isPressed && (
        <div className="absolute inset-0 bg-white/10 animate-pulse" />
      )}
    </Button>
  );
}