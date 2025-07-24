"use client";

import React, { createContext, useContext, useMemo } from "react";
import { LoadingSpinner } from "@/components/common/LoadingSpinner";
import { useLoadingState } from "./ui-context";

interface LoadingContextType {
  isLoading: boolean;
  startLoading: () => void;
  stopLoading: () => void;
}

const LoadingContext = createContext<LoadingContextType | undefined>(undefined);

export function LoadingProvider({ children }: { children: React.ReactNode }) {
  // Use the new UI context for loading state
  const { globalLoading, setLoading } = useLoadingState();
  
  const startLoading = () => setLoading(null, true);
  const stopLoading = () => setLoading(null, false);
  
  // Memoize context value
  const contextValue = useMemo(() => ({
    isLoading: globalLoading,
    startLoading,
    stopLoading
  }), [globalLoading, startLoading, stopLoading]);

  return (
    <LoadingContext.Provider value={contextValue}>
      {globalLoading && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
          <LoadingSpinner size="lg" className="text-primary" />
        </div>
      )}
      {children}
    </LoadingContext.Provider>
  );
}

export function useLoading() {
  const context = useContext(LoadingContext);
  if (context === undefined) {
    throw new Error("useLoading must be used within a LoadingProvider");
  }
  return context;
}

// Enhanced loading hook with request-specific loading states
export function useRequestLoading() {
  const { requestLoading, setLoading, isLoading } = useLoadingState();
  
  const setRequestLoading = (key: string, loading: boolean) => {
    setLoading(key, loading);
  };
  
  const isRequestLoading = (key: string) => {
    return requestLoading[key] || false;
  };
  
  return {
    setRequestLoading,
    isRequestLoading,
    isAnyLoading: isLoading(),
    activeRequests: Object.keys(requestLoading).filter(key => requestLoading[key])
  };
}
