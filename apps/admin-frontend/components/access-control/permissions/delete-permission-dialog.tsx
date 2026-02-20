import { Trash2 } from 'lucide-react';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import type { PermissionDefinition } from '@/lib/api/permissions-client';

export interface DeletePermissionDialogProps {
    permToDelete: PermissionDefinition | null;
    onClose: () => void;
    onConfirm: () => void;
}

export function DeletePermissionDialog({
    permToDelete,
    onClose,
    onConfirm,
}: DeletePermissionDialogProps) {
    return (
        <Dialog
            open={Boolean(permToDelete)}
            onOpenChange={open => {
                if (!open) {onClose();}
            }}
        >
            <DialogContent className="max-w-[400px]">
                <DialogHeader>
                    <div className="h-12 w-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4 mx-auto sm:mx-0">
                        <Trash2 className="h-6 w-6 text-red-500" />
                    </div>
                    <DialogTitle className="text-xl">Delete Permission?</DialogTitle>
                    <DialogDescription className="text-slate-400">
                        Are you sure you want to delete <span className="text-white font-bold">{permToDelete?.permission_string}</span>?
                        This action cannot be undone and may cause system errors if this permission is still in use.
                    </DialogDescription>
                </DialogHeader>
                <DialogFooter className="mt-4 gap-2 sm:gap-0">
                    <Button
                        variant="ghost"
                        onClick={onClose}
                        className="text-slate-400 hover:text-white hover:bg-gray-100 dark:hover:bg-white/5"
                    >
                        No, Keep it
                    </Button>
                    <Button
                        variant="destructive"
                        onClick={onConfirm}
                        className="bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/20"
                    >
                        Yes, Delete Permission
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
