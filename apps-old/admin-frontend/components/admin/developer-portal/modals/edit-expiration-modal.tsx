'use client';

import { Calendar } from 'lucide-react';
import React, { useState } from 'react';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';

interface EditExpirationModalProps {
    isOpen: boolean;
    onClose: () => void;
    apiKey: {
        id: string;
        client_name: string;
        expires_at?: string;
    };
    onUpdate: (keyId: string, expiresAt: string | null) => Promise<void>;
    isLoading?: boolean;
}

const KeyInfo: React.FC<{ clientName: string; expiresAt?: string }> = ({ clientName, expiresAt }) => (
    <div className="bg-gray-50 dark:bg-muted/50 rounded-lg p-4">
        <div className="text-sm">
            <div className="flex justify-between mb-1">
                <span className="text-muted-foreground">API Key:</span>
                <span className="font-medium text-foreground">
                    {clientName}
                </span>
            </div>
            <div className="flex justify-between">
                <span className="text-muted-foreground">Current Expiration:</span>
                <span className="font-medium text-foreground">
                    {expiresAt !== undefined && expiresAt !== ''
                        ? new Date(expiresAt).toLocaleDateString()
                        : 'Never'}
                </span>
            </div>
        </div>
    </div>
);

const getInitialDate = (expiresAt?: string): string => {
    if (expiresAt !== undefined && expiresAt !== '') {
        try {
            return new Date(expiresAt).toISOString().slice(0, 16);
        } catch {
            return '';
        }
    }
    return '';
};

const PresetButtons: React.FC<{ onSetPreset: (days: number) => void; disabled: boolean }> = ({ onSetPreset, disabled }) => (
    <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-muted-foreground mb-2">
            Quick Presets
        </label>
        <div className="flex flex-wrap gap-2">
            {[7, 30, 90, 365].map(days => (
                <Button
                    key={days}
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => onSetPreset(days)}
                    disabled={disabled}
                >
                    {days === 365 ? '1 Year' : `${days} Days`}
                </Button>
            ))}
        </div>
    </div>
);

export const EditExpirationModal: React.FC<EditExpirationModalProps> = ({
    isOpen,
    onClose,
    apiKey,
    onUpdate,
    isLoading = false,
}) => {
    const [expirationDate, setExpirationDate] = useState(() => getInitialDate(apiKey.expires_at));
    const [removeExpiration, setRemoveExpiration] = useState(false);
    const [error, setError] = useState('');

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');

        try {
            if (removeExpiration) {
                await onUpdate(apiKey.id, null);
            } else if (expirationDate !== '') {
                const isoDate = new Date(expirationDate).toISOString();
                await onUpdate(apiKey.id, isoDate);
            } else {
                setError('Please select an expiration date or choose to remove expiration');
                return;
            }
            onClose();
        } catch (_err) {
            setError('Failed to update expiration. Please try again.');
        }
    };

    const handleClose = () => {
        setExpirationDate(getInitialDate(apiKey.expires_at));
        setRemoveExpiration(false);
        setError('');
        onClose();
    };

    const setPreset = (days: number) => {
        const date = new Date();
        date.setDate(date.getDate() + days);
        setExpirationDate(date.toISOString().slice(0, 16));
        setRemoveExpiration(false);
    };

    return (
        <Dialog open={isOpen} onOpenChange={(open) => !open && handleClose()}>
            <DialogContent className="sm:max-w-md p-0 gap-0">
                <DialogHeader className="p-6 border-b border-gray-200 dark:border-border/40">
                    <DialogTitle className="flex items-center space-x-3">
                        <div className="p-2 bg-blue-100 dark:bg-blue-900/50 rounded-lg">
                            <Calendar className="w-5 h-5 text-blue-600 dark:text-blue-400" />
                        </div>
                        <span>Edit Expiration</span>
                    </DialogTitle>
                </DialogHeader>

                <form onSubmit={(e) => { void handleSubmit(e); }}>
                    <div className="p-6 space-y-4">
                        <KeyInfo clientName={apiKey.client_name} expiresAt={apiKey.expires_at} />
                        <PresetButtons onSetPreset={setPreset} disabled={removeExpiration} />

                        <div>
                            <label className="block text-sm font-medium text-gray-700 dark:text-muted-foreground mb-2">
                                Custom Expiration Date
                            </label>
                            <input
                                type="datetime-local"
                                value={expirationDate}
                                onChange={(e) => {
                                    setExpirationDate(e.target.value);
                                    setRemoveExpiration(false);
                                }}
                                disabled={removeExpiration}
                                min={new Date().toISOString().slice(0, 16)}
                                className="w-full px-3 py-2 border border-border/40 rounded-lg bg-white dark:bg-muted text-foreground focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                            />
                        </div>

                        <div className="flex items-center space-x-2">
                            <input
                                type="checkbox"
                                id="remove-expiration"
                                checked={removeExpiration}
                                onChange={(e) => {
                                    setRemoveExpiration(e.target.checked);
                                    if (e.target.checked) {
                                        setExpirationDate('');
                                    }
                                }}
                                className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                            />
                            <label
                                htmlFor="remove-expiration"
                                className="text-sm text-gray-700 dark:text-muted-foreground"
                            >
                                Remove expiration (key never expires)
                            </label>
                        </div>

                        {error !== '' && (
                            <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
                        )}
                    </div>

                    <DialogFooter className="p-6 border-t border-gray-200 dark:border-border/40 bg-gray-50 dark:bg-muted/50 sm:justify-end space-x-3 rounded-b-lg">
                        <Button
                            type="button"
                            variant="outline"
                            onClick={handleClose}
                            disabled={isLoading}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            disabled={isLoading || (expirationDate === '' && !removeExpiration)}
                        >
                            {isLoading ? 'Updating...' : 'Update Expiration'}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
};
