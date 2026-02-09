'use client';

import { Calendar, X } from 'lucide-react';
import React, { useState } from 'react';

import { Button } from '@/components/ui/button';

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
    <div className="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
        <div className="text-sm">
            <div className="flex justify-between mb-1">
                <span className="text-gray-500 dark:text-gray-400">API Key:</span>
                <span className="font-medium text-gray-900 dark:text-gray-100">
                    {clientName}
                </span>
            </div>
            <div className="flex justify-between">
                <span className="text-gray-500 dark:text-gray-400">Current Expiration:</span>
                <span className="font-medium text-gray-900 dark:text-gray-100">
                    {expiresAt !== undefined && expiresAt !== null && expiresAt !== ''
                        ? new Date(expiresAt).toLocaleDateString()
                        : 'Never'}
                </span>
            </div>
        </div>
    </div>
);

const PresetButtons: React.FC<{ onSetPreset: (days: number) => void; disabled: boolean }> = ({ onSetPreset, disabled }) => (
    <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
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
    const getInitialDate = () => {
        if (apiKey.expires_at !== undefined && apiKey.expires_at !== null && apiKey.expires_at !== '') {
            try {
                return new Date(apiKey.expires_at).toISOString().slice(0, 16);
            } catch {
                return '';
            }
        }
        return '';
    };

    const [expirationDate, setExpirationDate] = useState(getInitialDate());
    const [removeExpiration, setRemoveExpiration] = useState(false);
    const [error, setError] = useState('');

    if (!isOpen) { return null; }

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');

        try {
            if (removeExpiration) {
                await onUpdate(apiKey.id, null);
            } else if (expirationDate !== '') {
                // Convert to ISO format with timezone
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
        setExpirationDate(getInitialDate());
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
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4">
                <div className="p-6 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                        <div className="p-2 bg-blue-100 dark:bg-blue-900/50 rounded-lg">
                            <Calendar className="w-5 h-5 text-blue-600 dark:text-blue-400" />
                        </div>
                        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                            Edit Expiration
                        </h2>
                    </div>
                    <button
                        onClick={handleClose}
                        className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                <form onSubmit={(e) => { void handleSubmit(e); }}>
                    <div className="p-6 space-y-4">
                        <KeyInfo clientName={apiKey.client_name} expiresAt={apiKey.expires_at} />
                        <PresetButtons onSetPreset={setPreset} disabled={removeExpiration} />

                        <div>
                            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
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
                                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
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
                                className="text-sm text-gray-700 dark:text-gray-300"
                            >
                                Remove expiration (key never expires)
                            </label>
                        </div>

                        {error !== '' && (
                            <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
                        )}
                    </div>

                    <div className="p-6 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-700/50 flex justify-end space-x-3 rounded-b-lg">
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
                    </div>
                </form>
            </div>
        </div>
    );
};
