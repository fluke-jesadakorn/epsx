'use client';

import type { ReactNode} from 'react';
import { createContext, useContext, useEffect, useMemo, useState } from 'react';

interface NavbarContextType {
  isHydrated: boolean;
  isMobile: boolean;
}

const NavbarContext = createContext<NavbarContextType>({
  isHydrated: false,
  isMobile: false,
});

export const useNavbarContext = () => {
  return useContext(NavbarContext);
};

interface NavbarProviderProps {
  children: ReactNode;
}

export function NavbarProvider({ children }: NavbarProviderProps) {
  const [isHydrated, setIsHydrated] = useState(false);
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    // Single mount effect for entire navbar
    const checkMobile = () => setIsMobile(window.innerWidth < 768);
    checkMobile();

    // Safety timeout: ensure hydration completes even if rAF is blocked
    const safetyTimeout = setTimeout(() => {
      setIsHydrated(true);
    }, 100); // 100ms max wait

    // Use single requestAnimationFrame to prevent hydration mismatch
    // This ensures client matches server state initially, then hydrates
    requestAnimationFrame(() => {
      setIsHydrated(true);
      clearTimeout(safetyTimeout);
    });

    // Listen for resize events
    window.addEventListener('resize', checkMobile);
    return () => {
      window.removeEventListener('resize', checkMobile);
      clearTimeout(safetyTimeout);
    };
  }, []);

  const contextValue = useMemo(() => ({
    isHydrated,
    isMobile
  }), [isHydrated, isMobile]);

  return (
    <NavbarContext.Provider value={contextValue}>
      {children}
    </NavbarContext.Provider>
  );
}