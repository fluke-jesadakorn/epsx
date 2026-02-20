'use client';

import { Check, Loader2, Search } from 'lucide-react';
import { useMemo, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import type { AccessItem } from '@/hooks/use-wallet-access';
import { cn } from '@/lib/utils';

interface AddResourceModalProps {
    isOpen: boolean;
    onClose: () => void;
    title: string;
    description?: string;
    items: AccessItem[];
    onConfirm: (item: AccessItem) => Promise<void>;
    isLoading?: boolean;
    emptyMessage?: string;
}

export function AddResourceModal({
    isOpen,
    onClose,
    title,
    description,
    items,
    onConfirm,
    isLoading = false,
    emptyMessage = 'No items available'
}: AddResourceModalProps) {
    const [search, setSearch] = useState('');
    const [selectedId, setSelectedId] = useState<string | null>(null);

    const filteredItems = useMemo(() => {
        if (!search) { return items; }
        const lower = search.toLowerCase();
        return items.filter(item =>
            item.name.toLowerCase().includes(lower) ||
            item.description?.toLowerCase().includes(lower)
        );
    }, [items, search]);

    const handleConfirm = async () => {
        if (!selectedId) { return; }
        const item = items.find(i => i.id === selectedId);
        if (item) {
            await onConfirm(item);
            onClose();
            setSelectedId(null);
            setSearch('');
        }
    };

    return (
        <Dialog open={isOpen} onOpenChange={onClose}>
            <DialogContent className="sm:max-w-[500px] p-0 overflow-hidden">
                <DialogHeader className="px-6 py-4 border-b border-gray-100 dark:border-gray-800">
                    <DialogTitle>{title}</DialogTitle>
                    {description && <DialogDescription>{description}</DialogDescription>}
                </DialogHeader>

                <div className="p-4 space-y-4">
                    <div className="relative">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                        <Input
                            placeholder="Search..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            className="pl-9 bg-gray-50 dark:bg-slate-900/50"
                        />
                    </div>

                    <div className="max-h-[300px] overflow-y-auto space-y-1">
                        {filteredItems.length === 0 ? (
                            <div className="text-center py-8 text-gray-500 text-sm">
                                {emptyMessage}
                            </div>
                        ) : (
                            filteredItems.map(item => (
                                <button
                                    key={item.id}
                                    onClick={() => setSelectedId(item.id)}
                                    className={cn(
                                        "w-full flex items-center justify-between p-3 rounded-lg text-left transition-colors border",
                                        selectedId === item.id
                                            ? "bg-blue-50 border-blue-200 dark:bg-blue-900/20 dark:border-blue-800"
                                            : "bg-white dark:bg-gray-800 border-transparent hover:bg-gray-50 dark:hover:bg-gray-700 hover:border-gray-200 dark:hover:border-gray-600"
                                    )}
                                >
                                    <div>
                                        <p className="font-medium text-sm text-gray-900 dark:text-white">
                                            {item.name}
                                        </p>
                                        {item.description && (
                                            <p className="text-xs text-gray-500 truncate max-w-[300px]">
                                                {item.description}
                                            </p>
                                        )}
                                    </div>
                                    {selectedId === item.id && (
                                        <Check className="h-4 w-4 text-blue-600" />
                                    )}
                                </button>
                            ))
                        )}
                    </div>
                </div>

                <DialogFooter className="px-6 py-4 bg-gray-50 dark:bg-slate-900/50 border-t border-gray-100 dark:border-gray-800">
                    <Button variant="outline" onClick={onClose} disabled={isLoading}>
                        Cancel
                    </Button>
                    <Button
                        onClick={() => { void handleConfirm(); }}
                        disabled={!selectedId || isLoading}
                        className="gap-2"
                    >
                        {isLoading && <Loader2 className="h-4 w-4 animate-spin" />}
                        Add Selected
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
