
import { AlertCircle, CheckCircle, Info, XCircle } from 'lucide-react'
import React from 'react'
import type { NotificationType, NotificationVariant } from './unified-notification-types'

const TEXT_WHITE = 'text-white'

export const positions = {
    'top-right': 'top-4 right-4',
    'top-left': 'top-4 left-4',
    'bottom-right': 'bottom-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'top-center': 'top-4 left-1/2 -translate-x-1/2',
    'bottom-center': 'bottom-4 left-1/2 -translate-x-1/2'
}

export interface TypeStyle {
    lucideIcon: React.ReactNode
    emojiIcon: string
    accent?: string
    light: { bg: string; text: string }
    dark?: { bg: string; text: string }
}

export const typeStyles: Record<NotificationType, TypeStyle> = {
    info: {
        lucideIcon: <Info className="w-5 h-5 text-blue-500" />,
        emojiIcon: 'ℹ',
        accent: 'bg-blue-500',
        light: { bg: 'bg-blue-50 border-blue-200', text: 'text-blue-900' },
        dark: { bg: 'bg-blue-600', text: TEXT_WHITE }
    },
    success: {
        lucideIcon: <CheckCircle className="w-5 h-5 text-green-500" />,
        emojiIcon: '✓',
        accent: 'bg-green-500',
        light: { bg: 'bg-green-50 border-green-200', text: 'text-green-900' },
        dark: { bg: 'bg-green-600', text: TEXT_WHITE }
    },
    warning: {
        lucideIcon: <AlertCircle className="w-5 h-5 text-yellow-500" />,
        emojiIcon: '⚠',
        accent: 'bg-yellow-500',
        light: { bg: 'bg-yellow-50 border-yellow-200', text: 'text-yellow-900' },
        dark: { bg: 'bg-yellow-600', text: TEXT_WHITE }
    },
    error: {
        lucideIcon: <XCircle className="w-5 h-5 text-red-500" />,
        emojiIcon: '✕',
        accent: 'bg-red-500',
        light: { bg: 'bg-red-50 border-red-200', text: 'text-red-900' },
        dark: { bg: 'bg-red-600', text: TEXT_WHITE }
    }
}

export interface VariantStyle {
    container?: string
    icon?: string
    text?: string
    useDark?: boolean
    alertClass?: string
}

export const variantStyles: Record<NotificationVariant, VariantStyle> = {
    default: {
        container: 'shadow-lg bg-white border',
        icon: 'bg-gray-100',
        text: 'text-gray-900',
        useDark: false,
        alertClass: ''
    },
    pancake: {
        container: 'shadow-2xl bg-white border-l-4 border-orange-500 backdrop-blur-xl',
        icon: 'text-white rounded-none',
        text: 'text-gray-800',
        useDark: true,
        alertClass: 'shadow-lg'
    },
    admin: {
        container: 'shadow-2xl bg-slate-800 border-l-4 border-blue-500 backdrop-blur-xl',
        icon: `${TEXT_WHITE  } rounded-none`,
        text: TEXT_WHITE,
        useDark: true,
        alertClass: `shadow-lg bg-slate-800 ${  TEXT_WHITE}`
    },
    analytics: {
        container: 'shadow-lg backdrop-blur-sm bg-white/95 border',
        icon: 'bg-gray-100',
        text: 'text-gray-900',
        useDark: false,
        alertClass: 'backdrop-blur-sm bg-white/95'
    },
    premium: {
        container: 'shadow-xl backdrop-blur-md bg-gradient-to-r from-white to-blue-50/50 border',
        icon: 'bg-blue-100',
        text: 'text-gray-900',
        useDark: false,
        alertClass: 'shadow-md bg-gradient-to-r from-white to-blue-50/50'
    }
}
