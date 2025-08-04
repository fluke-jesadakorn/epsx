'use client';

import React, { createContext, useContext } from 'react';

// Simple module auth context - placeholder for future module-specific auth
const ModuleAuthContext = createContext<{} | null>({});

export function ModuleAuthProvider({ children }: { children: React.ReactNode }) {
  return (
    <ModuleAuthContext.Provider value={{}}>
      {children}
    </ModuleAuthContext.Provider>
  );
}

export function useModuleAuth() {
  const context = useContext(ModuleAuthContext);
  if (!context) {
    throw new Error('useModuleAuth must be used within ModuleAuthProvider');
  }
  return context;
}