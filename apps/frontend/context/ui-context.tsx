'use client'

import React, { createContext, useCallback, useContext, useMemo, useState } from 'react'

type ToastType = 'info' | 'success' | 'warning' | 'error'

type Toast = {
    id: string
    type: ToastType
    title?: string
    message: string
}

type ToastApi = {
    info: (message: string, title?: string) => void
    success: (message: string, title?: string) => void
    warning: (message: string, title?: string) => void
    error: (message: string, title?: string) => void
    dismiss: (id: string) => void
    clear: () => void
    toasts: Toast[]
}

const UIContext = createContext<ToastApi | null>(null)

function createToast(type: ToastType, message: string, title?: string): Toast {
    return {
        id: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
        type,
        title,
        message,
    }
}

export function UIProvider({ children }: { children: React.ReactNode }) {
    const [toasts, setToasts] = useState<Toast[]>([])

    const dismiss = useCallback((id: string) => {
        setToasts((prev) => prev.filter((t) => t.id !== id))
    }, [])

    const clear = useCallback(() => {
        setToasts([])
    }, [])

    const push = useCallback((type: ToastType, message: string, title?: string) => {
        const toast = createToast(type, message, title)
        setToasts((prev) => [...prev, toast])
    }, [])

    const api = useMemo<ToastApi>(() => {
        return {
            toasts,
            info: (message, title) => push('info', message, title),
            success: (message, title) => push('success', message, title),
            warning: (message, title) => push('warning', message, title),
            error: (message, title) => push('error', message, title),
            dismiss,
            clear,
        }
    }, [toasts, push, dismiss, clear])

    return <UIContext.Provider value={api}>{children}</UIContext.Provider>
}

export function useToasts() {
    const ctx = useContext(UIContext)
    if (!ctx) {
        throw new Error('useToasts must be used within a UIProvider')
    }
    return ctx
}
