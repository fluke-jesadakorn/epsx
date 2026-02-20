'use client';

import { Loader2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { updatePermissionAction } from '@/app/wallet-management/access/permission-actions';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetFooter,
    SheetHeader,
    SheetTitle,
} from '@/components/ui/sheet';
import { Textarea } from '@/components/ui/textarea';
import type { PermissionDefinition } from '@/lib/api/permissions-client';

interface Props {
    perm: PermissionDefinition | null;
    onOpenChange: (o: boolean) => void;
    onSuccess: () => void;
}

export function EditPermissionSheet({ perm, onOpenChange, onSuccess }: Props) {
    const [name, setName] = useState('');
    const [category, setCategory] = useState('');
    const [description, setDescription] = useState('');
    const [submitting, setSubmitting] = useState(false);

    useEffect(() => {
        if (perm) {
            setName(perm.name ?? '');
            setCategory(perm.category ?? '');
            setDescription(perm.description ?? '');
        }
    }, [perm]);

    const handleSubmit = async () => {
        if (!perm) return;
        setSubmitting(true);
        try {
            const res = await updatePermissionAction(perm.id, {
                name: name.trim() !== '' ? name.trim() : undefined,
                description: description.trim() !== '' ? description.trim() : undefined,
                category: category.trim() !== '' ? category.trim() : undefined,
            });
            if (res.success) {
                toast.success('Permission updated');
                onSuccess();
                onOpenChange(false);
            } else {
                toast.error(res.error ?? 'Failed to update permission');
            }
        } catch (err: unknown) {
            toast.error(err instanceof Error ? err.message : 'Failed to update');
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <Sheet open={perm !== null} onOpenChange={onOpenChange}>
            <SheetContent side="right" className="w-[400px] sm:w-[540px] bg-white dark:bg-card border-gray-200 dark:border-border text-white flex flex-col h-full">
                <SheetHeader>
                    <SheetTitle>Edit Permission</SheetTitle>
                    <SheetDescription className="font-mono text-xs text-cyan-400">
                        {perm?.permission_string}
                    </SheetDescription>
                </SheetHeader>
                <div className="space-y-6 pt-6 flex-1 flex flex-col overflow-y-auto">
                    <div className="space-y-2">
                        <Label>Name</Label>
                        <Input value={name} onChange={e => setName(e.target.value)} className="bg-white dark:bg-white/[0.04]" />
                    </div>
                    <div className="space-y-2">
                        <Label>Category</Label>
                        <Input value={category} onChange={e => setCategory(e.target.value)} className="bg-white dark:bg-white/[0.04]" />
                    </div>
                    <div className="space-y-2">
                        <Label>Description</Label>
                        <Textarea value={description} onChange={e => setDescription(e.target.value)} className="bg-white dark:bg-white/[0.04] min-h-[100px]" />
                    </div>
                    <SheetFooter className="mt-auto pt-6">
                        <Button type="button" disabled={submitting} onClick={() => void handleSubmit()} className="bg-cyan-500 w-full">
                            {submitting ? <Loader2 className="animate-spin w-4 h-4" /> : 'Save Changes'}
                        </Button>
                    </SheetFooter>
                </div>
            </SheetContent>
        </Sheet>
    );
}
