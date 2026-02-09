import { Trash2 } from 'lucide-react';

import { type AccessPolicy } from '@/components/access-control/types';
import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';

interface DeletePolicyModalProps {
    policy: AccessPolicy | null;
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    isDeleting: boolean;
}

export function DeletePolicyModal({
    policy,
    isOpen,
    onClose,
    onConfirm,
    isDeleting,
}: DeletePolicyModalProps) {
    if (!policy) {
        return null;
    }

    return (
        <Dialog open={isOpen} onOpenChange={onClose}>
            <DialogContent className="max-w-md bg-card border-border shadow-2xl">
                <DialogHeader>
                    <div className="flex items-center gap-3 mb-4">
                        <div className="p-3 bg-destructive/10 rounded-full">
                            <Trash2 className="w-6 h-6 text-destructive" />
                        </div>
                        <DialogTitle className="text-lg font-semibold text-foreground">
                            Delete Policy
                        </DialogTitle>
                    </div>
                    <DialogDescription className="text-muted-foreground">
                        Are you sure you want to delete &quot;<strong>{policy.name}</strong>
                        &quot;? This action cannot be undone.
                        {policy.memberCount > 0 && (
                            <span className="block mt-2 text-amber-600 dark:text-amber-400 font-medium">
                                ⚠️ This policy has {policy.memberCount} members who will lose
                                access.
                            </span>
                        )}
                    </DialogDescription>
                </DialogHeader>
                <DialogFooter className="flex gap-3 mt-6 sm:justify-between w-full">
                    <Button
                        variant="outline"
                        onClick={onClose}
                        className="flex-1"
                        disabled={isDeleting}
                    >
                        Cancel
                    </Button>
                    <Button
                        variant="destructive"
                        onClick={onConfirm}
                        disabled={isDeleting}
                        className="flex-1"
                    >
                        {isDeleting ? 'Deleting...' : 'Delete Policy'}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
