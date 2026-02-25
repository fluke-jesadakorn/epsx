'use client';

import { createContext, useContext, useTransition } from 'react';

interface TransitionCtx {
  pending: boolean;
  start: (fn: () => void) => void;
}

const Ctx = createContext<TransitionCtx>({
  pending: false,
  start: (fn) => fn(),
});

export function AnalyticsTransitionProvider({ children }: { children: React.ReactNode }) {
  const [pending, start] = useTransition();
  return <Ctx.Provider value={{ pending, start }}>{children}</Ctx.Provider>;
}

export function useAnalyticsTransition() {
  return useContext(Ctx);
}
