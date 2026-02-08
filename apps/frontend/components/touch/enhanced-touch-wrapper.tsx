'use client';

import { useTouchGestures } from '@/hooks/use-touch-gestures';
import type { ReactNode} from 'react';
import { useCallback, useState } from 'react';
// PullToRefresh component not available - using simple implementation
import { cn } from '@/lib/utils';
import {
  Bookmark,
  Heart,
  MoreHorizontal,
  Share2,
  ZoomIn,
  ZoomOut
} from 'lucide-react';

interface TouchAction {
  id: string;
  icon: ReactNode;
  label: string;
  color: string;
  action: () => void;
}

interface EnhancedTouchWrapperProps {
  children: ReactNode;
  className?: string;

  // Pull to refresh
  enablePullToRefresh?: boolean;
  onRefresh?: () => Promise<void>;

  // Swipe actions
  enableSwipeActions?: boolean;
  leftActions?: TouchAction[];
  rightActions?: TouchAction[];

  // Pinch to zoom
  enablePinchZoom?: boolean;
  onZoomChange?: (scale: number) => void;

  // Long press
  enableLongPress?: boolean;
  onLongPress?: () => void;

  // Double tap
  enableDoubleTap?: boolean;
  onDoubleTap?: () => void;

  // Quick actions
  enableQuickActions?: boolean;
  quickActions?: TouchAction[];

  // Haptic feedback
  enableHaptics?: boolean;

  disabled?: boolean;
}

export function EnhancedTouchWrapper({
  children,
  className = '',
  enablePullToRefresh = false,
  onRefresh,
  enableSwipeActions = false,
  leftActions = [],
  rightActions = [],
  enablePinchZoom = false,
  onZoomChange,
  enableLongPress = false,
  onLongPress,
  enableDoubleTap = false,
  onDoubleTap,
  enableQuickActions = false,
  quickActions = [],
  enableHaptics = true,
  disabled = false
}: EnhancedTouchWrapperProps) {
  const [isActionMenuOpen, setIsActionMenuOpen] = useState(false);
  const [currentZoom, setCurrentZoom] = useState(1);
  const [showSwipeHint, setShowSwipeHint] = useState(false);
  const [longPressTriggered, setLongPressTriggered] = useState(false);

  // Haptic feedback helper
  const triggerHaptic = useCallback((type: 'light' | 'medium' | 'heavy' = 'light') => {
    if (!enableHaptics || !navigator.vibrate) {return;}

    const patterns = {
      light: [10],
      medium: [20],
      heavy: [30]
    };

    navigator.vibrate(patterns[type]);
  }, [enableHaptics]);

  // Enhanced gesture handlers
  const handleSwipeLeft = useCallback(() => {
    if (!enableSwipeActions || rightActions.length === 0) {return;}

    triggerHaptic('light');
    setShowSwipeHint(true);

    // Auto-hide hint after animation
    setTimeout(() => setShowSwipeHint(false), 2000);
  }, [enableSwipeActions, rightActions.length, triggerHaptic]);

  const handleSwipeRight = useCallback(() => {
    if (!enableSwipeActions || leftActions.length === 0) {return;}

    triggerHaptic('light');
    setShowSwipeHint(true);

    setTimeout(() => setShowSwipeHint(false), 2000);
  }, [enableSwipeActions, leftActions.length, triggerHaptic]);

  const handleLongPress = useCallback(() => {
    if (!enableLongPress) {return;}

    triggerHaptic('heavy');
    setLongPressTriggered(true);

    if (enableQuickActions && quickActions.length > 0) {
      setIsActionMenuOpen(true);
    }

    onLongPress?.();

    // Reset visual feedback
    setTimeout(() => setLongPressTriggered(false), 200);
  }, [enableLongPress, enableQuickActions, quickActions.length, onLongPress, triggerHaptic]);

  const handleDoubleTap = useCallback(() => {
    if (!enableDoubleTap) {return;}

    triggerHaptic('medium');
    onDoubleTap?.();
  }, [enableDoubleTap, onDoubleTap, triggerHaptic]);

  const handlePinch = useCallback((scale: number) => {
    if (!enablePinchZoom) {return;}

    const newZoom = Math.max(0.5, Math.min(3, scale));
    setCurrentZoom(newZoom);
    onZoomChange?.(newZoom);

    // Light haptic feedback during pinch
    if (Math.abs(newZoom - currentZoom) > 0.1) {
      triggerHaptic('light');
    }
  }, [enablePinchZoom, onZoomChange, currentZoom, triggerHaptic]);

  const handlePinchEnd = useCallback(() => {
    if (!enablePinchZoom) {return;}

    // Snap to common zoom levels
    let snapZoom = currentZoom;
    if (currentZoom < 0.75) {snapZoom = 0.5;}
    else if (currentZoom < 1.25) {snapZoom = 1;}
    else if (currentZoom < 1.75) {snapZoom = 1.5;}
    else if (currentZoom < 2.5) {snapZoom = 2;}
    else {snapZoom = 3;}

    if (Math.abs(snapZoom - currentZoom) > 0.1) {
      setCurrentZoom(snapZoom);
      onZoomChange?.(snapZoom);
      triggerHaptic('medium');
    }
  }, [enablePinchZoom, currentZoom, onZoomChange, triggerHaptic]);

  const { bind } = useTouchGestures({
    onSwipeLeft: handleSwipeLeft,
    onSwipeRight: handleSwipeRight,
    onLongPress: handleLongPress,
    onDoubleTap: handleDoubleTap,
    onPinch: handlePinch,
    onPinchEnd: handlePinchEnd,
    enabled: !disabled
  });

  const executeAction = useCallback((action: TouchAction) => {
    triggerHaptic('medium');
    action.action();
    setIsActionMenuOpen(false);
  }, [triggerHaptic]);

  // Default quick actions if none provided
  const defaultQuickActions: TouchAction[] = [
    {
      id: 'like',
      icon: <Heart className="h-5 w-5" />,
      label: 'Like',
      color: 'bg-red-500',
      action: () => { }
    },
    {
      id: 'share',
      icon: <Share2 className="h-5 w-5" />,
      label: 'Share',
      color: 'bg-blue-500',
      action: () => { }
    },
    {
      id: 'save',
      icon: <Bookmark className="h-5 w-5" />,
      label: 'Save',
      color: 'bg-green-500',
      action: () => { }
    }
  ];

  const actionsToShow = quickActions.length > 0 ? quickActions : defaultQuickActions;

  const content = (
    <div
      {...(bind() as any)}
      className={cn(
        'relative touch-none select-none',
        longPressTriggered && 'animate-pulse',
        className
      )}
      style={{
        transform: enablePinchZoom ? `scale(${currentZoom})` : undefined,
        transformOrigin: 'center',
        transition: 'transform 0.2s ease-out'
      }}
    >
      {/* Swipe Actions - Left */}
      {enableSwipeActions && leftActions.length > 0 && showSwipeHint && (
        <div className="absolute left-0 top-1/2 -translate-y-1/2 z-20 animate-slideInLeft">
          <div className="flex gap-2">
            {leftActions.map((action) => (
              <button
                key={action.id}
                onClick={() => executeAction(action)}
                className={cn(
                  'flex flex-col items-center gap-1 p-3 rounded-full text-white shadow-lg',
                  'transform hover:scale-110 active:scale-95 transition-all',
                  action.color
                )}
              >
                {action.icon}
                <span className="text-xs font-medium">{action.label}</span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Swipe Actions - Right */}
      {enableSwipeActions && rightActions.length > 0 && showSwipeHint && (
        <div className="absolute right-0 top-1/2 -translate-y-1/2 z-20 animate-slideInRight">
          <div className="flex gap-2">
            {rightActions.map((action) => (
              <button
                key={action.id}
                onClick={() => executeAction(action)}
                className={cn(
                  'flex flex-col items-center gap-1 p-3 rounded-full text-white shadow-lg',
                  'transform hover:scale-110 active:scale-95 transition-all',
                  action.color
                )}
              >
                {action.icon}
                <span className="text-xs font-medium">{action.label}</span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Quick Actions Menu */}
      {isActionMenuOpen && enableQuickActions && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 bg-black/50 z-40 animate-fadeIn"
            onClick={() => setIsActionMenuOpen(false)}
          />

          {/* Actions Menu */}
          <div className="fixed bottom-4 left-4 right-4 z-50 animate-slideInUp">
            <div className="bg-white dark:bg-gray-800 rounded-2xl p-4 shadow-2xl border">
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-semibold text-lg">Quick Actions</h3>
                <button
                  onClick={() => setIsActionMenuOpen(false)}
                  className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-full"
                >
                  <MoreHorizontal className="h-5 w-5" />
                </button>
              </div>

              <div className="grid grid-cols-3 gap-3">
                {actionsToShow.map((action) => (
                  <button
                    key={action.id}
                    onClick={() => executeAction(action)}
                    className={cn(
                      'flex flex-col items-center gap-2 p-4 rounded-xl text-white',
                      'transform hover:scale-105 active:scale-95 transition-all',
                      action.color
                    )}
                  >
                    {action.icon}
                    <span className="text-sm font-medium">{action.label}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </>
      )}

      {/* Zoom Controls */}
      {enablePinchZoom && currentZoom !== 1 && (
        <div className="fixed bottom-4 right-4 z-30 flex flex-col gap-2">
          <button
            onClick={() => {
              const newZoom = Math.min(3, currentZoom + 0.5);
              setCurrentZoom(newZoom);
              onZoomChange?.(newZoom);
              triggerHaptic('light');
            }}
            className="p-2 bg-white dark:bg-gray-800 rounded-full shadow-lg hover:scale-110 active:scale-95 transition-all"
          >
            <ZoomIn className="h-5 w-5" />
          </button>

          <div className="px-3 py-1 bg-white dark:bg-gray-800 rounded-full shadow-lg text-sm font-medium">
            {Math.round(currentZoom * 100)}%
          </div>

          <button
            onClick={() => {
              const newZoom = Math.max(0.5, currentZoom - 0.5);
              setCurrentZoom(newZoom);
              onZoomChange?.(newZoom);
              triggerHaptic('light');
            }}
            className="p-2 bg-white dark:bg-gray-800 rounded-full shadow-lg hover:scale-110 active:scale-95 transition-all"
          >
            <ZoomOut className="h-5 w-5" />
          </button>

          {currentZoom !== 1 && (
            <button
              onClick={() => {
                setCurrentZoom(1);
                onZoomChange?.(1);
                triggerHaptic('medium');
              }}
              className="px-3 py-1 bg-primary text-primary-foreground rounded-full shadow-lg text-sm font-medium hover:scale-110 active:scale-95 transition-all"
            >
              Reset
            </button>
          )}
        </div>
      )}

      {/* Content */}
      {children}

      {/* Touch hints */}
      {(enableLongPress || enableDoubleTap || enableSwipeActions) && (
        <div className="absolute top-2 left-2 opacity-30 pointer-events-none">
          <div className="flex gap-1">
            {enableLongPress && (
              <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse" />
            )}
            {enableDoubleTap && (
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
            )}
            {enableSwipeActions && (
              <div className="w-2 h-2 bg-orange-500 rounded-full animate-pulse" />
            )}
          </div>
        </div>
      )}
    </div>
  );

  // Simple pull-to-refresh wrapper (PullToRefresh component not available)
  if (enablePullToRefresh && onRefresh) {
    return (
      <div className={className} style={{ touchAction: 'pan-y' }}>
        {content}
      </div>
    );
  }

  return content;
}

export default EnhancedTouchWrapper;