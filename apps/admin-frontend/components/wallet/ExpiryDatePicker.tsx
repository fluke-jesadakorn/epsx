'use client';

import { Calendar, Clock, X } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface ExpiryDatePickerProps {
    itemName: string;
    itemType: 'permission' | 'group' | 'plan';
    isOpen: boolean;
    onConfirm: (expiresAt: Date | null) => void;
    onCancel: () => void;
}

const PRESETS = [
    { label: '7 days', days: 7 },
    { label: '30 days', days: 30 },
    { label: '90 days', days: 90 },
    { label: '1 year', days: 365 },
];

/**
 *
 * @param root0
 * @param root0.itemName
 * @param root0.itemType
 * @param root0.isOpen
 * @param root0.onConfirm
 * @param root0.onCancel
 */
export function ExpiryDatePicker({
    itemName,
    itemType,
    isOpen,
    onConfirm,
    onCancel,
}: ExpiryDatePickerProps) {
    const [selectedDate, setSelectedDate] = useState<Date | null>(null);
    const [customDate, setCustomDate] = useState('');

    if (!isOpen) { return null; }

    const handlePreset = (days: number) => {
        const date = new Date();
        date.setDate(date.getDate() + days);
        setSelectedDate(date);
        const dateStr = date.toISOString().split('T')[0];
        setCustomDate(dateStr ?? '');
    };

    const handleCustomDateChange = (value: string) => {
        setCustomDate(value);
        if (value) {
            setSelectedDate(new Date(value));
        } else {
            setSelectedDate(null);
        }
    };

    const handleConfirm = () => {
        onConfirm(selectedDate);
        setSelectedDate(null);
        setCustomDate('');
    };

    const handleNoExpiry = () => {
        onConfirm(null);
        setSelectedDate(null);
        setCustomDate('');
    };

    const handleCancel = () => {
        onCancel();
        setSelectedDate(null);
        setCustomDate('');
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
            {/* Backdrop */}
            <div
                className="absolute inset-0 bg-black/50 backdrop-blur-sm"
                onClick={handleCancel}
            />

            {/* Modal */}
            <div className="relative bg-white dark:bg-gray-900 rounded-2xl shadow-2xl border border-gray-200 dark:border-gray-700 w-full max-w-md mx-4 overflow-hidden">
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-800">
                    <div className="flex items-center gap-3">
                        <div className="p-2 rounded-full bg-blue-100 dark:bg-blue-900/30">
                            <Calendar className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                        </div>
                        <div>
                            <h3 className="font-semibold text-gray-900 dark:text-white">
                                Set Expiry Date
                            </h3>
                            <p className="text-sm text-gray-500 dark:text-gray-400">
                                Assigning {itemType}: <span className="font-medium text-gray-700 dark:text-gray-300">{itemName}</span>
                            </p>
                        </div>
                    </div>
                    <button
                        onClick={handleCancel}
                        className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                    >
                        <X className="h-5 w-5 text-gray-400" />
                    </button>
                </div>

                {/* Content */}
                <div className="p-4 space-y-4">
                    {/* Quick Presets */}
                    <div>
                        <label className="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2 block">
                            Quick Presets
                        </label>
                        <div className="grid grid-cols-4 gap-2">
                            {PRESETS.map((preset) => (
                                <button
                                    key={preset.days}
                                    onClick={() => handlePreset(preset.days)}
                                    className={cn(
                                        'px-3 py-2 rounded-lg text-sm font-medium transition-all',
                                        selectedDate && customDate === new Date(Date.now() + preset.days * 86400000).toISOString().split('T')[0]
                                            ? 'bg-blue-600 text-white'
                                            : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                                    )}
                                >
                                    {preset.label}
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Custom Date */}
                    <div>
                        <label className="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2 block">
                            Custom Date
                        </label>
                        <div className="relative">
                            <Clock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                            <input
                                type="date"
                                value={customDate}
                                onChange={(e) => handleCustomDateChange(e.target.value)}
                                min={new Date().toISOString().split('T')[0]}
                                className="w-full pl-10 pr-4 py-2.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                            />
                        </div>
                    </div>

                    {/* Selected Preview */}
                    {selectedDate && (
                        <div className="p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
                            <p className="text-sm text-blue-700 dark:text-blue-300">
                                <span className="font-medium">Expires:</span>{' '}
                                {selectedDate.toLocaleDateString('en-US', {
                                    weekday: 'long',
                                    year: 'numeric',
                                    month: 'long',
                                    day: 'numeric',
                                })}
                            </p>
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div className="flex items-center justify-between p-4 border-t border-gray-100 dark:border-gray-800 bg-gray-50 dark:bg-gray-800/50">
                    <Button
                        variant="ghost"
                        onClick={handleNoExpiry}
                        className="text-gray-600 dark:text-gray-400"
                    >
                        No Expiry
                    </Button>
                    <div className="flex gap-2">
                        <Button
                            variant="outline"
                            onClick={handleCancel}
                        >
                            Cancel
                        </Button>
                        <Button
                            onClick={handleConfirm}
                            disabled={!selectedDate}
                            className="bg-blue-600 hover:bg-blue-700"
                        >
                            Confirm
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default ExpiryDatePicker;
