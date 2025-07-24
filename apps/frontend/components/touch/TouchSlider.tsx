'use client';

import { ReactNode, useRef, useState, useCallback, useEffect } from 'react';

interface TouchSliderProps {
  children: ReactNode[];
  initialIndex?: number;
  onChange?: (index: number) => void;
  autoplay?: boolean;
  autoplayInterval?: number;
  showIndicators?: boolean;
  showArrows?: boolean;
  className?: string;
  slideClassName?: string;
}

export function TouchSlider({
  children,
  initialIndex = 0,
  onChange,
  autoplay = false,
  autoplayInterval = 3000,
  showIndicators = true,
  showArrows = false,
  className = '',
  slideClassName = ''
}: TouchSliderProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [currentIndex, setCurrentIndex] = useState(initialIndex);
  const [isDragging, setIsDragging] = useState(false);
  const [startX, setStartX] = useState(0);
  const [translateX, setTranslateX] = useState(0);
  const autoplayRef = useRef<NodeJS.Timeout>();

  const slideCount = children.length;

  const goToSlide = useCallback((index: number) => {
    const newIndex = Math.max(0, Math.min(index, slideCount - 1));
    setCurrentIndex(newIndex);
    onChange?.(newIndex);
  }, [slideCount, onChange]);

  const nextSlide = useCallback(() => {
    goToSlide(currentIndex + 1);
  }, [currentIndex, goToSlide]);

  const prevSlide = useCallback(() => {
    goToSlide(currentIndex - 1);
  }, [currentIndex, goToSlide]);

  // Autoplay functionality
  useEffect(() => {
    if (autoplay && !isDragging) {
      autoplayRef.current = setInterval(() => {
        setCurrentIndex(prev => {
          const next = prev + 1;
          if (next >= slideCount) {
            return 0; // Loop back to first slide
          }
          return next;
        });
      }, autoplayInterval);
    }

    return () => {
      if (autoplayRef.current) {
        clearInterval(autoplayRef.current);
      }
    };
  }, [autoplay, autoplayInterval, isDragging, slideCount]);

  // Touch event handlers
  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    setIsDragging(true);
    setStartX(e.touches[0].clientX);
    setTranslateX(0);
    
    // Stop autoplay when user starts dragging
    if (autoplayRef.current) {
      clearInterval(autoplayRef.current);
    }
  }, []);

  const handleTouchMove = useCallback((e: React.TouchEvent) => {
    if (!isDragging) return;
    
    const currentX = e.touches[0].clientX;
    const deltaX = currentX - startX;
    setTranslateX(deltaX);
  }, [isDragging, startX]);

  const handleTouchEnd = useCallback(() => {
    if (!isDragging) return;
    
    setIsDragging(false);
    
    const threshold = 50; // Minimum distance to trigger slide change
    
    if (Math.abs(translateX) > threshold) {
      if (translateX > 0) {
        // Swipe right - go to previous slide
        prevSlide();
      } else {
        // Swipe left - go to next slide
        nextSlide();
      }
    }
    
    setTranslateX(0);
  }, [isDragging, translateX, nextSlide, prevSlide]);

  // Mouse event handlers for desktop
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    setIsDragging(true);
    setStartX(e.clientX);
    setTranslateX(0);
  }, []);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isDragging) return;
    
    const currentX = e.clientX;
    const deltaX = currentX - startX;
    setTranslateX(deltaX);
  }, [isDragging, startX]);

  const handleMouseUp = useCallback(() => {
    if (!isDragging) return;
    
    setIsDragging(false);
    
    const threshold = 50;
    
    if (Math.abs(translateX) > threshold) {
      if (translateX > 0) {
        prevSlide();
      } else {
        nextSlide();
      }
    }
    
    setTranslateX(0);
  }, [isDragging, translateX, nextSlide, prevSlide]);

  const getTransform = () => {
    const baseTransform = -currentIndex * 100;
    const dragTransform = isDragging ? (translateX / (containerRef.current?.offsetWidth || 1)) * 100 : 0;
    return `translateX(${baseTransform + dragTransform}%)`;
  };

  return (
    <div className={`relative overflow-hidden ${className}`}>
      {/* Slides Container */}
      <div
        ref={containerRef}
        className="flex transition-transform duration-300 ease-out"
        style={{
          transform: getTransform(),
          transitionDuration: isDragging ? '0ms' : '300ms'
        }}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        {children.map((child, index) => (
          <div
            key={index}
            className={`flex-shrink-0 w-full ${slideClassName}`}
          >
            {child}
          </div>
        ))}
      </div>

      {/* Navigation Arrows */}
      {showArrows && (
        <>
          <button
            onClick={prevSlide}
            disabled={currentIndex === 0}
            className="absolute left-2 top-1/2 -translate-y-1/2 w-10 h-10 rounded-full bg-background/80 backdrop-blur-sm border shadow-lg flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed hover:bg-background transition-colors z-10"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          
          <button
            onClick={nextSlide}
            disabled={currentIndex === slideCount - 1}
            className="absolute right-2 top-1/2 -translate-y-1/2 w-10 h-10 rounded-full bg-background/80 backdrop-blur-sm border shadow-lg flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed hover:bg-background transition-colors z-10"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </>
      )}

      {/* Indicators */}
      {showIndicators && (
        <div className="absolute bottom-4 left-1/2 -translate-x-1/2 flex gap-2 z-10">
          {Array.from({ length: slideCount }).map((_, index) => (
            <button
              key={index}
              onClick={() => goToSlide(index)}
              className={`w-2 h-2 rounded-full transition-all duration-200 ${
                index === currentIndex
                  ? 'bg-primary w-6'
                  : 'bg-white/50 hover:bg-white/70'
              }`}
            />
          ))}
        </div>
      )}
    </div>
  );
}