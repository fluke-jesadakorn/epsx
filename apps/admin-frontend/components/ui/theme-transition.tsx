'use client';
import { useEffect } from 'react';

/**
 *
 */
export function ThemeTransition() {
  useEffect(() => {
    // Remove preload class after initial page load to enable transitions
    const timer = setTimeout(() => {
      document.documentElement.classList.remove('preload');
    }, 100);

    return () => clearTimeout(timer);
  }, []);

  return null;
}