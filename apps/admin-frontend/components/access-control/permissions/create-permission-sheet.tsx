import { Loader2 } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';

import { createPermissionAction } from '@/app/wallet-management/access/permission-actions';
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
import type { CreatePermissionRequest } from '@/lib/api/permissions-client';

const emptyForm: CreatePermissionRequest = { permission: '', name: '', description: '', platform: '', category: '' };

export function CreatePermissionSheet({
    open,
    onOpenChange,
    onSuccess,
}: {
    open: boolean;
    onOpenChange: (o: boolean) => void;
    onSuccess: () => void;
}) {
    const [formData, setFormData] = useState<CreatePermissionRequest>(emptyForm);
    const [submitting, setSubmitting] = useState(false);

    const handleSubmit = async () => {
        if (formData.permission.trim() === '') {
            toast.error('Permission string is required');
            return;
        }
        setSubmitting(true);
        try {
            const payload: CreatePermissionRequest = {
                permission: formData.permission.trim(),
                name: formData.name?.trim() !== '' ? formData.name?.trim() : undefined,
                description: formData.description?.trim() !== '' ? formData.description?.trim() : undefined,
                platform: formData.platform?.trim() !== '' ? formData.platform?.trim() : undefined,
                category: formData.category?.trim() !== '' ? formData.category?.trim() : undefined,
            };
            const res = await createPermissionAction(payload);
            if (res.success) {
                toast.success('Permission created');
                setFormData(emptyForm);
                onSuccess();
                onOpenChange(false);
            } else {
                toast.error(res.error ?? 'Failed to create permission');
            }
        } catch (err: unknown) {
            toast.error(err instanceof Error ? err.message : 'Failed to create permission');
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetContent side="right" className="w-[400px] sm:w-[540px] bg-white dark:bg-card border-gray-200 dark:border-border text-white flex flex-col h-full">
                <SheetHeader>
                    <SheetTitle>Create Permission</SheetTitle>
                    <SheetDescription>Define a new system permission.</SheetDescription>
                </SheetHeader>
                <div className="space-y-6 pt-6 flex-1 flex flex-col overflow-y-auto">
                    <div className="space-y-2">
                        <Label>Permission String *</Label>
                        <Input
                            placeholder="e.g. admin:users:delete"
                            value={formData.permission}
                            onChange={e => setFormData({ ...formData, permission: e.target.value })}
                            className="font-mono bg-white dark:bg-white/[0.04]"
                        />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                            <Label>Name</Label>
                            <Input value={formData.name} onChange={e => setFormData({ ...formData, name: e.target.value })} className="bg-white dark:bg-white/[0.04]" />
                        </div>
                        <div className="space-y-2">
                            <Label>Platform</Label>
                            <Input
                                placeholder="e.g. admin, epsx"
                                value={formData.platform}
                                onChange={e => setFormData({ ...formData, platform: e.target.value })}
                                className="bg-white dark:bg-white/[0.04]"
                            />
                        </div>
                    </div>
                    <div className="space-y-2">
                        <Label>Category</Label>
                        <Input value={formData.category} onChange={e => setFormData({ ...formData, category: e.target.value })} className="bg-white dark:bg-white/[0.04]" />
                    </div>
                    <div className="space-y-2">
                        <Label>Description</Label>
                        <Textarea value={formData.description} onChange={e => setFormData({ ...formData, description: e.target.value })} className="bg-white dark:bg-white/[0.04] min-h-[100px]" />
                    </div>
                    <SheetFooter className="mt-auto pt-6">
                        <Button type="button" disabled={submitting} onClick={() => void handleSubmit()} className="bg-emerald-500 w-full">
                            {submitting ? <Loader2 className="animate-spin w-4 h-4" /> : 'Create permission'}
                        </Button>
                    </SheetFooter>
                </div>
            </SheetContent>
        </Sheet>
    );
}
