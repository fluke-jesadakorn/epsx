'use client';

import { useRef, useCallback, useState } from 'react';

interface TouchGestureOptions {
  onSwipeLeft?: () => void;
  onSwipeRight?: () => void;
  onSwipeUp?: () => void;
  onSwipeDown?: () => void;
  onPinch?: (scale: number) => void;
  onPinchStart?: () => void;
  onPinchEnd?: () => void;
  onPanStart?: () => void;
  onPanMove?: (deltaX: number, deltaY: number) => void;
  onPanEnd?: () => void;
  onTap?: () => void;
  onDoubleTap?: () => void;
  onLongPress?: () => void;
  swipeThreshold?: number;
  pinchThreshold?: number;
  longPressDelay?: number;
  doubleTapDelay?: number;
  enabled?: boolean;
}

interface TouchState {
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
  startTime: number;
  touchCount: number;
  initialDistance: number;
  currentScale: number;
}

export function useTouchGestures(options: TouchGestureOptions = {}) {
  const {
    onSwipeLeft,
    onSwipeRight,
    onSwipeUp,
    onSwipeDown,
    onPinch,
    onPinchStart,
    onPinchEnd,
    onPanStart,
    onPanMove,
    onPanEnd,
    onTap,
    onDoubleTap,
    onLongPress,
    swipeThreshold = 50,
    pinchThreshold = 0.1,
    longPressDelay = 500,
    doubleTapDelay = 300,
    enabled = true
  } = options;

  const touchState = useRef<TouchState>({
    startX: 0,
    startY: 0,
    currentX: 0,
    currentY: 0,
    startTime: 0,
    touchCount: 0,
    initialDistance: 0,
    currentScale: 1
  });

  const longPressTimer = useRef<NodeJS.Timeout | null>(null);
  const lastTapTime = useRef(0);
  const [isPanning, setIsPanning] = useState(false);
  const [isPinching, setIsPinching] = useState(false);
  const [transform, setTransform] = useState('');

  const clearTimers = useCallback(() => {
    if (longPressTimer.current) {
      clearTimeout(longPressTimer.current);
      longPressTimer.current = null;
    }
  }, []);

  const getDistance = useCallback((touch1: Touch, touch2: Touch): number => {
    const deltaX = touch1.clientX - touch2.clientX;
    const deltaY = touch1.clientY - touch2.clientY;
    return Math.sqrt(deltaX * deltaX + deltaY * deltaY);
  }, []);

  const handleTouchStart = useCallback((e: TouchEvent) => {
    if (!enabled) return;

    const touch = e.touches[0];
    const state = touchState.current;

    state.startX = touch.clientX;
    state.startY = touch.clientY;
    state.currentX = touch.clientX;
    state.currentY = touch.clientY;
    state.startTime = Date.now();
    state.touchCount = e.touches.length;

    if (e.touches.length === 2) {
      // Pinch gesture start
      const touch2 = e.touches[1];
      state.initialDistance = getDistance(touch, touch2);
      state.currentScale = 1;
      setIsPinching(true);
      onPinchStart?.();
    } else if (e.touches.length === 1) {
      // Single touch - could be pan, tap, or long press
      setIsPanning(true);
      onPanStart?.();

      // Start long press timer
      if (onLongPress) {
        longPressTimer.current = setTimeout(() => {
          onLongPress();
        }, longPressDelay);
      }
    }
  }, [enabled, onPinchStart, onPanStart, onLongPress, longPressDelay, getDistance]);

  const handleTouchMove = useCallback((e: TouchEvent) => {
    if (!enabled) return;

    const touch = e.touches[0];
    const state = touchState.current;

    state.currentX = touch.clientX;
    state.currentY = touch.clientY;

    const deltaX = state.currentX - state.startX;
    const deltaY = state.currentY - state.startY;

    // Clear long press timer on movement
    if (Math.abs(deltaX) > 10 || Math.abs(deltaY) > 10) {
      clearTimers();
    }

    if (e.touches.length === 2 && isPinching) {
      // Pinch gesture
      const touch2 = e.touches[1];
      const currentDistance = getDistance(touch, touch2);
      const scale = currentDistance / state.initialDistance;
      
      if (Math.abs(scale - state.currentScale) > pinchThreshold) {
        state.currentScale = scale;
        onPinch?.(scale);
      }
    } else if (e.touches.length === 1 && isPanning) {
      // Pan gesture
      onPanMove?.(deltaX, deltaY);
      setTransform(`translate(${deltaX}px, ${deltaY}px)`);
    }
  }, [enabled, isPinching, isPanning, onPinch, onPanMove, pinchThreshold, getDistance, clearTimers]);

  const handleTouchEnd = useCallback((_e: TouchEvent) => {
    if (!enabled) return;

    const state = touchState.current;
    const deltaX = state.currentX - state.startX;
    const deltaY = state.currentY - state.startY;
    const deltaTime = Date.now() - state.startTime;

    clearTimers();

    if (isPinching) {
      setIsPinching(false);
      onPinchEnd?.();
    }

    if (isPanning) {
      setIsPanning(false);
      onPanEnd?.();
      setTransform('');
    }

    // Check for swipe gestures
    const absX = Math.abs(deltaX);
    const absY = Math.abs(deltaY);

    if (absX > swipeThreshold || absY > swipeThreshold) {
      if (absX > absY) {
        // Horizontal swipe
        if (deltaX > 0) {
          onSwipeRight?.();
        } else {
          onSwipeLeft?.();
        }
      } else {
        // Vertical swipe
        if (deltaY > 0) {
          onSwipeDown?.();
        } else {
          onSwipeUp?.();
        }
      }
      return;
    }

    // Check for tap gestures
    if (deltaTime < 200 && absX < 10 && absY < 10) {
      const now = Date.now();
      
      if (now - lastTapTime.current < doubleTapDelay) {
        // Double tap
        onDoubleTap?.();
        lastTapTime.current = 0; // Reset to prevent triple tap
      } else {
        // Single tap
        setTimeout(() => {
          if (now === lastTapTime.current) {
            onTap?.();
          }
        }, doubleTapDelay);
        lastTapTime.current = now;
      }
    }
  }, [
    enabled,
    isPinching,
    isPanning,
    onPinchEnd,
    onPanEnd,
    onSwipeLeft,
    onSwipeRight,
    onSwipeUp,
    onSwipeDown,
    onTap,
    onDoubleTap,
    swipeThreshold,
    doubleTapDelay,
    clearTimers
  ]);

  const bind = useCallback(() => {
    if (!enabled) return {};

    return {
      onTouchStart: handleTouchStart,
      onTouchMove: handleTouchMove,
      onTouchEnd: handleTouchEnd,
      style: {
        touchAction: 'none',
        userSelect: 'none',
        WebkitUserSelect: 'none'
      } as React.CSSProperties
    };
  }, [enabled, handleTouchStart, handleTouchMove, handleTouchEnd]);

  return {
    bind,
    transform,
    isPanning,
    isPinching
  };
}