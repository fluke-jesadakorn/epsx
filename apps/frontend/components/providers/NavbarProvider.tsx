'use client';

import { createContext, ReactNode, useContext, useEffect, useMemo, useState } from 'react';

interface NavbarContextType {
  isHydrated: boolean;
  isMobile: boolean;
}

const NavbarContext = createContext<NavbarContextType>({
  isHydrated: false,
  isMobile: false,
});

export const useNavbarContext = () => {
  const context = useContext(NavbarContext);
  if (!context) {
    throw new Error('useNavbarContext must be used within NavbarProvider');
  }
  return context;
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

    // Use single requestAnimationFrame to prevent hydration mismatch
    // This ensures client matches server state initially, then hydrates
    requestAnimationFrame(() => {
      setIsHydrated(true);
    });

    // Listen for resize events
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
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