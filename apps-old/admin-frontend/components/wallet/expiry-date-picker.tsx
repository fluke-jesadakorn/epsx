'use client';

import { Calendar, Clock, X } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface ExpiryDatePickerProps {
    itemName: string;
    itemType: 'permission' | 'group' | 'plan' | 'items';
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
interface PickerHeaderProps {
    itemName: string;
    itemType: string;
    onCancel: () => void;
}

function PickerHeader({ itemName, itemType, onCancel }: PickerHeaderProps) {
    return (
        <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-800">
            <div className="flex items-center gap-3">
                <div className="p-2 rounded-full bg-blue-100 dark:bg-blue-900/30">
                    <Calendar className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                </div>
                <div>
                    <h3 className="font-semibold text-foreground">Set Expiry Date</h3>
                    <p className="text-sm text-muted-foreground">
                        Assigning {itemType}: <span className="font-medium text-gray-700 dark:text-muted-foreground">{itemName}</span>
                    </p>
                </div>
            </div>
            <button onClick={onCancel} className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
                <X className="h-5 w-5 text-gray-400" />
            </button>
        </div>
    );
}

interface PickerContentProps {
    selectedDate: Date | null;
    customDate: string;
    onPreset: (days: number) => void;
    onCustomChange: (v: string) => void;
}

function PickerContent({ selectedDate, customDate, onPreset, onCustomChange }: PickerContentProps) {
    return (
        <div className="p-4 space-y-4">
            <div>
                <label className="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2 block">Quick Presets</label>
                <div className="grid grid-cols-4 gap-2">
                    {PRESETS.map((preset) => (
                        <button
                            key={preset.days}
                            onClick={() => onPreset(preset.days)}
                            className={cn(
                                'px-3 py-2 rounded-lg text-sm font-medium transition-all',
                                selectedDate !== null && customDate === new Date(Date.now() + preset.days * 86400000).toISOString().split('T')[0]
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-muted/30 text-gray-700 dark:text-muted-foreground hover:bg-gray-200 dark:hover:bg-gray-700'
                            )}
                        >
                            {preset.label}
                        </button>
                    ))}
                </div>
            </div>
            <div>
                <label className="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2 block">Custom Date</label>
                <div className="relative">
                    <Clock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                    <input
                        type="date"
                        value={customDate}
                        onChange={(e) => onCustomChange(e.target.value)}
                        min={new Date().toISOString().split('T')[0]}
                        className="w-full pl-10 pr-4 py-2.5 rounded-lg border border-gray-200 dark:border-border/40 bg-card text-foreground focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                    />
                </div>
            </div>
            {selectedDate !== null && (
                <div className="p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
                    <p className="text-sm text-blue-700 dark:text-blue-300">
                        <span className="font-medium">Expires:</span>{' '}
                        {selectedDate.toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}
                    </p>
                </div>
            )}
        </div>
    );
}

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
        setCustomDate(date.toISOString().split('T')[0] ?? '');
    };

    const handleCustomDateChange = (value: string) => {
        setCustomDate(value);
        setSelectedDate(value !== '' ? new Date(value) : null);
    };

    const reset = () => { setSelectedDate(null); setCustomDate(''); };

    const handleConfirm = () => { onConfirm(selectedDate); reset(); };
    const handleNoExpiry = () => { onConfirm(null); reset(); };
    const handleCancel = () => { onCancel(); reset(); };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
            <div className="absolute inset-0 bg-black/50" onClick={handleCancel} />
            <div className="relative bg-card rounded-2xl shadow-2xl border border-gray-200 dark:border-border/40 w-full max-w-md mx-4 overflow-hidden">
                <PickerHeader itemName={itemName} itemType={itemType} onCancel={handleCancel} />
                <PickerContent selectedDate={selectedDate} customDate={customDate} onPreset={handlePreset} onCustomChange={handleCustomDateChange} />
                <div className="flex items-center justify-between p-4 border-t border-gray-100 dark:border-gray-800 bg-gray-50 dark:bg-muted/50">
                    <Button variant="ghost" onClick={handleNoExpiry} className="text-muted-foreground">No Expiry</Button>
                    <div className="flex gap-2">
                        <Button variant="outline" onClick={handleCancel}>Cancel</Button>
                        <Button onClick={handleConfirm} disabled={selectedDate === null} className="bg-blue-600 hover:bg-blue-700">Confirm</Button>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default ExpiryDatePicker;
