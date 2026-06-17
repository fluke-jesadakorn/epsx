'use client';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { AlertTriangle, Copy } from 'lucide-react';
import React from 'react';

interface NewApiKeyModalProps {
    apiKey: string;
    onClose: () => void;
    onCopy: (text: string, label: string) => void;
}

export const NewApiKeyModal: React.FC<NewApiKeyModalProps> = ({
    apiKey,
    onClose,
    onCopy,
}) => {
    return (
        <Dialog open={true} onOpenChange={(open) => !open && onClose()}>
            <DialogContent className="sm:max-w-md p-0 gap-0">
                <DialogHeader className="p-6 border-b border-border/40">
                    <DialogTitle>API Key Created</DialogTitle>
                </DialogHeader>

                <div className="p-6">
                    <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
                        <div className="flex items-start">
                            <AlertTriangle className="w-5 h-5 text-yellow-600 mt-0.5 mr-2" />
                            <div>
                                <h3 className="font-medium text-yellow-800">Important</h3>
                                <p className="text-sm text-yellow-700 mt-1">
                                    This is the only time you&apos;ll see your API key. Please
                                    copy it and store it securely.
                                </p>
                            </div>
                        </div>
                    </div>

                    <div className="bg-gray-50 dark:bg-muted rounded-lg p-4">
                        <div className="flex items-center justify-between mb-2">
                            <label className="text-sm font-medium text-gray-700 dark:text-muted-foreground">
                                Your API Key
                            </label>
                            <button
                                onClick={() => onCopy(apiKey, 'API Key')}
                                className="p-1 text-muted-foreground hover:text-gray-600 dark:hover:text-foreground"
                            >
                                <Copy className="w-4 h-4" />
                            </button>
                        </div>
                        <code className="block text-sm font-mono text-foreground bg-card p-3 rounded border border-border/40 break-all">
                            {apiKey}
                        </code>
                    </div>
                </div>

                <DialogFooter className="p-6 border-t border-border/40 bg-gray-50 dark:bg-muted sm:justify-end">
                    <Button onClick={onClose}>
                        I&apos;ve Saved the Key
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
};
