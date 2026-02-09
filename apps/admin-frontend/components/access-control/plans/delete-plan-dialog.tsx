import { Trash2 } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { type PermissionPlan } from '@/lib/api/plan-management-client';

export interface DeletePlanDialogProps {
    planToDelete: PermissionPlan | null;
    onClose: () => void;
    onConfirm: () => void;
}

export function DeletePlanDialog({
    planToDelete,
    onClose,
    onConfirm,
}: DeletePlanDialogProps) {
    const [deleteConfirmationInput, setDeleteConfirmationInput] = useState('');

    return (
        <Dialog
            open={Boolean(planToDelete)}
            onOpenChange={(open) => {
                if (!open) {
                    onClose();
                    setDeleteConfirmationInput('');
                }
            }}
        >
            <DialogContent className="max-w-[400px]">
                <DialogHeader>
                    <div className="h-12 w-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4 mx-auto sm:mx-0">
                        <Trash2 className="h-6 w-6 text-red-500" />
                    </div>
                    <DialogTitle className="text-xl">Delete Plan?</DialogTitle>
                    <DialogDescription className="text-slate-400">
                        Are you sure you want to delete{' '}
                        <span className="text-white font-bold">{planToDelete?.name}</span>?
                        This action cannot be undone and will affect all users currently
                        assigned to this plan.
                    </DialogDescription>
                    <div className="mt-4">
                        <Label className="text-xs text-slate-500 mb-2 block tracking-wider font-bold">
                            Type{' '}
                            <span className="text-white select-all">
                                {planToDelete?.name}
                            </span>{' '}
                            to confirm
                        </Label>
                        <Input
                            value={deleteConfirmationInput}
                            onChange={(e) => setDeleteConfirmationInput(e.target.value)}
                            placeholder="Type plan name"
                            className="bg-white/5 border-white/10 text-white placeholder:text-white/20"
                        />
                    </div>
                </DialogHeader>
                <DialogFooter className="mt-4 gap-2 sm:gap-0">
                    <Button
                        variant="ghost"
                        onClick={onClose}
                        className="text-slate-400 hover:text-white hover:bg-white/5"
                    >
                        Cancel
                    </Button>
                    <Button
                        variant="destructive"
                        onClick={onConfirm}
                        disabled={deleteConfirmationInput !== planToDelete?.name}
                        className="bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        Yes, Delete Plan
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
