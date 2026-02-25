'use client';

import { AlertTriangle } from 'lucide-react';
import React, { useState } from 'react';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';

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
// eslint-disable-next-line max-lines-per-function
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
        } catch (_err) {
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
        <Dialog open={isOpen} onOpenChange={(open) => !open && handleClose()}>
            <DialogContent className="sm:max-w-md p-0 gap-0">
                <DialogHeader className="p-6 border-b border-gray-200 dark:border-border/40">
                    <DialogTitle className="flex items-center space-x-3">
                        <div className="p-2 bg-red-100 dark:bg-red-900/50 rounded-lg">
                            <AlertTriangle className="w-5 h-5 text-red-600 dark:text-red-400" />
                        </div>
                        <span>Revoke API Key</span>
                    </DialogTitle>
                </DialogHeader>

                {/* eslint-disable-next-line @typescript-eslint/no-misused-promises */}
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
                        <div className="bg-gray-50 dark:bg-muted/50 rounded-lg p-4">
                            <div className="text-sm">
                                <div className="flex justify-between mb-1">
                                    <span className="text-muted-foreground">Client Name:</span>
                                    <span className="font-medium text-foreground">
                                        {apiKey.client_name}
                                    </span>
                                </div>
                                {/* eslint-disable-next-line @typescript-eslint/strict-boolean-expressions */}
                                {apiKey.wallet_address && (
                                    <div className="flex justify-between">
                                        <span className="text-muted-foreground">Wallet:</span>
                                        <span className="font-mono text-xs text-foreground">
                                            {apiKey.wallet_address.slice(0, 6)}...{apiKey.wallet_address.slice(-4)}
                                        </span>
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* Reason Selection */}
                        <div>
                            <label className="block text-sm font-medium text-gray-700 dark:text-muted-foreground mb-2">
                                Reason for Revocation <span className="text-red-500">*</span>
                            </label>
                            <select
                                value={selectedReason}
                                onChange={(e) => setSelectedReason(e.target.value)}
                                className="w-full px-3 py-2 border border-border/40 rounded-lg bg-white dark:bg-muted text-foreground focus:ring-2 focus:ring-blue-500 focus:border-transparent"
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
                                <label className="block text-sm font-medium text-gray-700 dark:text-muted-foreground mb-2">
                                    Specify Reason <span className="text-red-500">*</span>
                                </label>
                                <textarea
                                    value={customReason}
                                    onChange={(e) => setCustomReason(e.target.value)}
                                    placeholder="Enter the reason for revocation..."
                                    rows={3}
                                    className="w-full px-3 py-2 border border-border/40 rounded-lg bg-white dark:bg-muted text-foreground placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                                    required
                                />
                            </div>
                        )}

                        {/* Error */}
                        {error && (
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
                            variant="destructive"
                            disabled={isLoading || !selectedReason}
                            className="bg-red-600 hover:bg-red-700 text-white"
                        >
                            {isLoading ? 'Revoking...' : 'Revoke API Key'}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
};
