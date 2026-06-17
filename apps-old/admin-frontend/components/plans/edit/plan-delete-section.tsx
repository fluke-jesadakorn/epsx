'use client'

import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from '@/components/ui/alert-dialog'

interface PlanDeleteSectionProps {
    planName: string
    onDelete: () => Promise<void>
}

export function PlanDeleteSection({
    planName,
    onDelete,
}: PlanDeleteSectionProps) {
    return (
        <div className="mt-12 pt-8 border-t border-gray-200 dark:border-border/40">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 p-6 bg-red-50 dark:bg-red-900/10 rounded-2xl border border-red-100 dark:border-red-900/30">
                <div>
                    <h3 className="text-lg font-bold text-red-700 dark:text-red-400">
                        Danger Zone
                    </h3>
                    <p className="text-sm text-red-600/80 dark:text-red-400/70 mt-1">
                        Deleting this plan will remove it from the system. This action cannot
                        be undone.
                    </p>
                </div>
                <div>
                    <AlertDialog>
                        <AlertDialogTrigger asChild>
                            <button
                                type="button"
                                className="px-6 py-3 rounded-xl font-semibold bg-white border-2 border-red-200 text-red-600 hover:bg-red-50 dark:bg-transparent dark:border-red-800 dark:text-red-400 dark:hover:bg-red-900/20 transition-colors"
                                aria-label="Delete Plan Trigger"
                            >
                                Delete Plan
                            </button>
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                            <AlertDialogHeader>
                                <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
                                <AlertDialogDescription>
                                    This action cannot be undone. This will permanently delete the
                                    <span className="font-bold text-foreground">
                                        {' '}
                                        &quot;{planName}&quot;{' '}
                                    </span>
                                    plan and remove it from our servers.
                                </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogCancel>Cancel</AlertDialogCancel>
                                <AlertDialogAction
                                    className="bg-red-600 hover:bg-red-700 text-white"
                                    onClick={() => void onDelete()}
                                >
                                    Delete Plan
                                </AlertDialogAction>
                            </AlertDialogFooter>
                        </AlertDialogContent>
                    </AlertDialog>
                </div>
            </div>
        </div>
    )
}
