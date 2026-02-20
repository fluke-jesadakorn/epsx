'use client';

import { useCallback, useState } from 'react';
import { toast } from 'sonner';
import { copyToClipboard as copyToClipboardUtil } from '@/lib/utils';

export function useDocumentationExpanded() {
    const [expandedModule, setExpandedModule] = useState<string | null>(null);
    const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null);

    const toggleModule = useCallback((moduleName: string) => {
        setExpandedModule((prev) => (prev === moduleName ? null : moduleName));
    }, []);

    const toggleEndpoint = useCallback((endpointKey: string) => {
        setExpandedEndpoint((prev) => (prev === endpointKey ? null : endpointKey));
    }, []);

    return { expandedModule, setExpandedModule, expandedEndpoint, setExpandedEndpoint, toggleModule, toggleEndpoint };
}

export function useCopyToClipboard() {
    const copyToClipboard = useCallback(async (text: string, label: string) => {
        const success = await copyToClipboardUtil(text);
        if (success) {
            toast.success(`${label} copied to clipboard`);
        } else {
            toast.error('Failed to copy to clipboard');
        }
    }, []);

    return { copyToClipboard };
}

export function useMethodColor() {
    return useCallback((method: string) => {
        switch (method.toUpperCase()) {
            case 'GET':
                return 'bg-green-100 text-green-800';
            case 'POST':
                return 'bg-blue-100 text-blue-800';
            case 'PUT':
                return 'bg-yellow-100 text-yellow-800';
            case 'DELETE':
                return 'bg-red-100 text-red-800';
            default:
                return 'bg-gray-100 text-gray-800';
        }
    }, []);
}

export function useAccessLevelColor() {
    return useCallback((level: string) => {
        if (level.includes('Bronze')) {
            return 'text-amber-600';
        }
        if (level.includes('Silver')) {
            return 'text-gray-500';
        }
        if (level.includes('Gold')) {
            return 'text-yellow-500';
        }
        if (level.includes('Platinum')) {
            return 'text-purple-600';
        }
        return 'text-blue-600';
    }, []);
}
