'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2 } from 'lucide-react';
import React, { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import {
    Form,
    FormControl,
    FormDescription,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/components/ui/use-toast';
import { walletMgmt } from '@/lib/api/wallet-management-client';
import { logger } from '@/lib/logger';

const formSchema = z.object({
    label: z.string().max(20, 'Label must be 20 characters or less').optional(),
    note: z.string().max(500, 'Note must be 500 characters or less').optional(),
});

interface EditWalletMetadataModalProps {
    walletAddress: string;
    currentLabel?: string;
    currentNote?: string;
    isOpen: boolean;
    onClose: () => void;
    onSuccess: () => void;
}

/**
 *
 * @param root0
 * @param root0.walletAddress
 * @param root0.currentLabel
 * @param root0.currentNote
 * @param root0.isOpen
 * @param root0.onClose
 * @param root0.onSuccess
 */
export function EditWalletMetadataModal({
    walletAddress,
    currentLabel,
    currentNote,
    isOpen,
    onClose,
    onSuccess,
}: EditWalletMetadataModalProps) {
    const [isLoading, setIsLoading] = React.useState(false);

    type FormValues = z.infer<typeof formSchema>;
    const form = useForm<FormValues>({
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        resolver: zodResolver(formSchema as any),
        defaultValues: {
            label: currentLabel ?? '',
            note: currentNote ?? '',
        },
    });

    // Reset form when opening for a different wallet
    useEffect(() => {
        if (isOpen) {
            form.reset({
                label: currentLabel ?? '',
                note: currentNote ?? '',
            });
        }
    }, [isOpen, currentLabel, currentNote, form]);

    const onSubmit = async (values: FormValues) => {
        setIsLoading(true);
        try {
            await walletMgmt.updateWalletMetadata(walletAddress, {
                label: values.label ?? null,
                note: values.note ?? null,
            });

            toast({
                title: 'Wallet updated',
                description: 'Label and note have been saved successfully.',
            });

            onSuccess();
            onClose();
        } catch (error) {
            logger.error('Failed to update wallet metadata:', { error });
            toast({
                title: 'Error',
                description: 'Failed to update wallet. Please try again.',
                variant: 'destructive',
            });
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Edit Wallet Details</DialogTitle>
                    <DialogDescription>
                        Add a label or note to organize and track this wallet.
                    </DialogDescription>
                </DialogHeader>

                {/* eslint-disable-next-line @typescript-eslint/no-explicit-any */}
                <Form {...(form as any)}>
                    <form onSubmit={(e) => { void form.handleSubmit(onSubmit)(e); }} className="space-y-4">
                        <FormField
                            // eslint-disable-next-line @typescript-eslint/no-explicit-any
                            control={form.control as any}
                            name="label"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>Label</FormLabel>
                                    <FormControl>
                                        <Input placeholder="e.g. VIP, Whale, Suspicious" {...field} />
                                    </FormControl>
                                    <FormDescription>
                                        Short tag for quick identification (max 20 chars).
                                    </FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <FormField
                            // eslint-disable-next-line @typescript-eslint/no-explicit-any
                            control={form.control as any}
                            name="note"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>Note</FormLabel>
                                    <FormControl>
                                        <Textarea
                                            placeholder="Add details about this wallet..."
                                            className="resize-none"
                                            rows={4}
                                            {...field}
                                        />
                                    </FormControl>
                                    <FormDescription>
                                        Internal notes for administrative record.
                                    </FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={onClose} disabled={isLoading}>
                                Cancel
                            </Button>
                            <Button type="submit" disabled={isLoading}>
                                {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                Save Changes
                            </Button>
                        </DialogFooter>
                    </form>
                </Form>
            </DialogContent>
        </Dialog>
    );
}
