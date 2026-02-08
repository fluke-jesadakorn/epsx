'use client'

import React, { createContext, useContext, useMemo, useState } from 'react'

type RootState = Record<string, unknown>

interface AppStateContextValue {
    state: RootState
    setState: React.Dispatch<React.SetStateAction<RootState>>
}

const AppStateContext = createContext<AppStateContextValue | null>(null)

export function AppStateProvider({
    children,
    initialState,
}: {
    children: React.ReactNode
    initialState?: RootState
}) {
    const [state, setState] = useState<RootState>(() => initialState ?? {})

    const value = useMemo(() => ({ state, setState }), [state])

    return <AppStateContext.Provider value={value}>{children}</AppStateContext.Provider>
}

export function useAppState() {
    const ctx = useContext(AppStateContext)
    if (!ctx) {
        throw new Error('useAppState must be used within an Appstate-provider')
    }
    return ctx
}
