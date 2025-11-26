'use client';

import React, { createContext, useContext, useCallback, useEffect, useMemo } from 'react';
import { useAppState } from './app-state';
import { Toast, UIState } from '@/lib/state/types';
import { useClientOnly } from '@/lib/state';

interface UIContextType {
  // Theme
  theme: UIState['theme'];
  setTheme: (theme: UIState['theme']) => void;
  
  // Sidebar
  sidebarOpen: boolean;
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  collapseSidebar: (collapsed: boolean) => void;
  
  // Modals
  modals: UIState['modals'];
  openModal: (key: string, data?: any) => void;
  closeModal: (key: string) => void;
  isModalOpen: (key: string) => boolean;
  
  // Toasts
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id' | 'timestamp'>) => void;
  removeToast: (id: string) => void;
  success: (title: string, description?: string) => void;
  error: (title: string, description?: string) => void;
  warning: (title: string, description?: string) => void;
  info: (title: string, description?: string) => void;
  
  // Loading
  globalLoading: boolean;
  requestLoading: Record<string, boolean>;
  setLoading: (key: string | null, loading: boolean) => void;
  isLoading: (key?: string) => boolean;
  
  // Responsive
  responsive: UIState['responsive'];
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
}

const UIContext = createContext<UIContextType | undefined>(undefined);

interface UIProviderProps {
  children: React.ReactNode;
}

export function UIProvider({ children }: UIProviderProps) {
  const { state, actions } = useAppState();
  const { ui } = state;

  // Responsive handling
  const updateResponsive = useCallback(() => {
    if (typeof window === 'undefined') return;
    
    const width = window.innerWidth;
    let breakpoint: UIState['responsive']['breakpoint'] = 'xs';
    
    if (width >= 1536) breakpoint = '2xl';
    else if (width >= 1280) breakpoint = 'xl';
    else if (width >= 1024) breakpoint = 'lg';
    else if (width >= 768) breakpoint = 'md';
    else if (width >= 640) breakpoint = 'sm';
    
    actions.ui.setResponsive({
      isMobile: width < 768,
      isTablet: width >= 768 && width < 1024,
      breakpoint
    });
  }, [actions.ui.setResponsive]);

  useEffect(() => {
    updateResponsive();
    window.addEventListener('resize', updateResponsive);
    return () => window.removeEventListener('resize', updateResponsive);
  }, []);

  // Auto-remove toasts
  useEffect(() => {
    const timers = ui.toasts
      .filter(toast => toast.duration !== 0) // Don't auto-remove persistent toasts
      .map(toast => {
        const duration = toast.duration || 5000;
        return setTimeout(() => {
          actions.ui.removeToast(toast.id);
        }, duration);
      });

    return () => {
      timers.forEach(timer => clearTimeout(timer));
    };
  }, [ui.toasts, actions.ui]);

  // Toast convenience methods
  const toastMethods = useMemo(() => ({
    success: (title: string, description?: string) => 
      actions.ui.addToast({ type: 'success', title, description }),
    error: (title: string, description?: string) => 
      actions.ui.addToast({ type: 'error', title, description, duration: 8000 }),
    warning: (title: string, description?: string) => 
      actions.ui.addToast({ type: 'warning', title, description }),
    info: (title: string, description?: string) => 
      actions.ui.addToast({ type: 'info', title, description })
  }), [actions.ui]);

  // Modal helper
  const isModalOpen = useCallback((key: string) => {
    return ui.modals[key]?.open || false;
  }, [ui.modals]);

  // Loading helper
  const isLoading = useCallback((key?: string) => {
    if (key) {
      return ui.loading.requests[key] || false;
    }
    return ui.loading.global || Object.values(ui.loading.requests).some(Boolean);
  }, [ui.loading]);

  // SSR-safe responsive values
  const hasMounted = useClientOnly();
  const isMobile = hasMounted ? ui.responsive.isMobile : false;
  const isTablet = hasMounted ? ui.responsive.isTablet : false;
  const isDesktop = hasMounted ? (!ui.responsive.isMobile && !ui.responsive.isTablet) : true;

  const contextValue = useMemo(() => ({
    // Theme
    theme: ui.theme,
    setTheme: actions.ui.setTheme,
    
    // Sidebar
    sidebarOpen: ui.sidebar.open,
    sidebarCollapsed: ui.sidebar.collapsed,
    toggleSidebar: actions.ui.toggleSidebar,
    collapseSidebar: actions.ui.collapseSidebar,
    
    // Modals
    modals: ui.modals,
    openModal: actions.ui.openModal,
    closeModal: actions.ui.closeModal,
    isModalOpen,
    
    // Toasts
    toasts: ui.toasts,
    addToast: actions.ui.addToast,
    removeToast: actions.ui.removeToast,
    ...toastMethods,
    
    // Loading
    globalLoading: ui.loading.global,
    requestLoading: ui.loading.requests,
    setLoading: actions.ui.setLoading,
    isLoading,
    
    // Responsive
    responsive: ui.responsive,
    isMobile,
    isTablet,
    isDesktop
  }), [
    ui,
    actions.ui,
    toastMethods,
    isModalOpen,
    isLoading,
    isMobile,
    isTablet,
    isDesktop
  ]);

  return (
    <UIContext.Provider value={contextValue}>
      {children}
    </UIContext.Provider>
  );
}

export function useUI() {
  const context = useContext(UIContext);
  if (context === undefined) {
    throw new Error('useUI must be used within a UIProvider');
  }
  return context;
}

// Specialized hooks
export function useTheme() {
  const { theme, setTheme } = useUI();
  return { theme, setTheme };
}

export function useSidebar() {
  const { sidebarOpen, sidebarCollapsed, toggleSidebar, collapseSidebar } = useUI();
  return { sidebarOpen, sidebarCollapsed, toggleSidebar, collapseSidebar };
}

export function useModals() {
  const { modals, openModal, closeModal, isModalOpen } = useUI();
  return { modals, openModal, closeModal, isModalOpen };
}

export function useToasts() {
  const { toasts, addToast, removeToast, success, error, warning, info } = useUI();
  return { toasts, addToast, removeToast, success, error, warning, info };
}

export function useLoadingState() {
  const { globalLoading, requestLoading, setLoading, isLoading } = useUI();
  return { globalLoading, requestLoading, setLoading, isLoading };
}

export function useResponsive() {
  const { responsive, isMobile, isTablet, isDesktop } = useUI();
  return { responsive, isMobile, isTablet, isDesktop };
}