'use client';

import { AlertTriangle, X } from 'lucide-react';
import React, { useState } from 'react';

import { Button } from '@/components/ui/button';

interface RevokeKeyModalProps {
    isOpen: boolean;
    onClose: () => void;
    apiKey: {
        id: string;
        client_name: string;
        wallet_address?: string;
    };
    onRevoke: (keyId: string, reason: string) => Promise<void>;
    isLoading?: boolean;
}

const COMMON_REASONS = [
    'Security concern - potential key exposure',
    'User requested revocation',
    'Suspicious activity detected',
    'Account suspended',
    'Violation of terms of service',
    'Key no longer needed',
    'Other (specify below)',
];

/**
 * Modal dialog for revoking an API key with reason
 * @param root0
 * @param root0.isOpen
 * @param root0.onClose
 * @param root0.apiKey
 * @param root0.onRevoke
 * @param root0.isLoading
 */
export const RevokeKeyModal: React.FC<RevokeKeyModalProps> = ({
    isOpen,
    onClose,
    apiKey,
    onRevoke,
    isLoading = false,
}) => {
    const [selectedReason, setSelectedReason] = useState('');
    const [customReason, setCustomReason] = useState('');
    const [error, setError] = useState('');

    if (!isOpen) {return null;}

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');

        const reason = selectedReason === 'Other (specify below)'
            ? customReason.trim()
            : selectedReason;

        if (!reason) {
            setError('Please provide a reason for revocation');
            return;
        }

        try {
            await onRevoke(apiKey.id, reason);
            onClose();
        } catch (err) {
            setError('Failed to revoke API key. Please try again.');
        }
    };

    const handleClose = () => {
        setSelectedReason('');
        setCustomReason('');
        setError('');
        onClose();
    };

    return (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4">
                {/* Header */}
                <div className="p-6 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                        <div className="p-2 bg-red-100 dark:bg-red-900/50 rounded-lg">
                            <AlertTriangle className="w-5 h-5 text-red-600 dark:text-red-400" />
                        </div>
                        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                            Revoke API Key
                        </h2>
                    </div>
                    <button
                        onClick={handleClose}
                        className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Body */}
                <form onSubmit={handleSubmit}>
                    <div className="p-6 space-y-4">
                        {/* Warning */}
                        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                            <p className="text-sm text-red-800 dark:text-red-200">
                                <strong>Warning:</strong> This action cannot be undone. The API key for{' '}
                                <span className="font-semibold">&quot;{apiKey.client_name}&quot;</span> will be
                                permanently revoked and cannot be used for authentication.
                            </p>
                        </div>

                        {/* Key Info */}
                        <div className="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
                            <div className="text-sm">
                                <div className="flex justify-between mb-1">
                                    <span className="text-gray-500 dark:text-gray-400">Client Name:</span>
                                    <span className="font-medium text-gray-900 dark:text-gray-100">
                                        {apiKey.client_name}
                                    </span>
                                </div>
                                {apiKey.wallet_address && (
                                    <div className="flex justify-between">
                                        <span className="text-gray-500 dark:text-gray-400">Wallet:</span>
                                        <span className="font-mono text-xs text-gray-900 dark:text-gray-100">
                                            {apiKey.wallet_address.slice(0, 6)}...{apiKey.wallet_address.slice(-4)}
                                        </span>
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* Reason Selection */}
                        <div>
                            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Reason for Revocation <span className="text-red-500">*</span>
                            </label>
                            <select
                                value={selectedReason}
                                onChange={(e) => setSelectedReason(e.target.value)}
                                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                required
                            >
                                <option value="">Select a reason...</option>
                                {COMMON_REASONS.map((reason) => (
                                    <option key={reason} value={reason}>
                                        {reason}
                                    </option>
                                ))}
                            </select>
                        </div>

                        {/* Custom Reason */}
                        {selectedReason === 'Other (specify below)' && (
                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Specify Reason <span className="text-red-500">*</span>
                                </label>
                                <textarea
                                    value={customReason}
                                    onChange={(e) => setCustomReason(e.target.value)}
                                    placeholder="Enter the reason for revocation..."
                                    rows={3}
                                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                                    required
                                />
                            </div>
                        )}

                        {/* Error */}
                        {error && (
                            <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
                        )}
                    </div>

                    {/* Footer */}
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
                            variant="destructive"
                            disabled={isLoading ?? !selectedReason}
                            className="bg-red-600 hover:bg-red-700 text-white"
                        >
                            {isLoading ? 'Revoking...' : 'Revoke API Key'}
                        </Button>
                    </div>
                </form>
            </div>
        </div>
    );
};
