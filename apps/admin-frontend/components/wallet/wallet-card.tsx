'use client';

import React, { useState } from 'react';

import { logger } from '@/lib/logger';
import { cn, copyToClipboard } from '@/lib/utils';
import type { WalletData } from './types';
import { WalletCardActions, WalletCardIdentity, WalletCardStats } from './wallet-card-sections';

interface WalletCardProps {
    wallet: WalletData;
    isSelected?: boolean;
    onSelect?: (selected: boolean) => void;
    onView?: () => void;
    onManage?: () => void;
    onDisable?: () => void;
    onEnable?: () => void;
    onEdit?: () => void;
    onUpdateMetadata?: (label: string | null, note: string | null) => Promise<void>;
    className?: string;
}

export function WalletCard({
    wallet,
    isSelected = false,
    onSelect,
    onView,
    onManage: _onManage,
    onDisable: _onDisable,
    onEnable,
    onEdit: _onEdit,
    onUpdateMetadata,
    className,
}: WalletCardProps) {
    const [copied, setCopied] = useState(false);
    const [isEditing, setIsEditing] = useState(false);
    const [labelInput, setLabelInput] = useState(wallet.label ?? '');
    const [noteInput, setNoteInput] = useState(wallet.note ?? '');
    const [isSaving, setIsSaving] = useState(false);

    const isDisabled = wallet.status === 'disabled';

    const handleCopy = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const success = await copyToClipboard(wallet.walletAddress);
        if (success) {
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const handleStartEditing = () => {
        setLabelInput(wallet.label ?? '');
        setNoteInput(wallet.note ?? '');
        setIsEditing(true);
    };

    const handleSaveMetadata = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (!onUpdateMetadata) { return; }
        setIsSaving(true);
        try {
            await onUpdateMetadata(labelInput || null, noteInput || null);
            setIsEditing(false);
        } catch (error) {
            logger.error('Failed to save metadata', { error });
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <div
            className={cn(
                "group relative w-full overflow-hidden rounded-[24px] border border-gray-200 dark:border-slate-700 bg-white/60 dark:bg-[#0f172a]/60 p-1 backdrop-blur-xl transition-all duration-300 hover:border-[#7645d9]/30 hover:shadow-2xl hover:shadow-[#7645d9]/10",
                isSelected && 'ring-2 ring-[#1fc7d4] bg-[#1fc7d4]/5',
                isDisabled && 'opacity-60 grayscale-[0.5]',
                className
            )}
        >
            <div className="absolute -left-16 -top-16 h-32 w-32 rounded-full bg-[#1fc7d4]/5 blur-[50px] transition-all duration-500 group-hover:bg-[#1fc7d4]/10" />
            <div className="absolute -right-16 -bottom-16 h-32 w-32 rounded-full bg-[#7645d9]/5 blur-[50px] transition-all duration-500 group-hover:bg-[#7645d9]/10" />

            <div className="relative flex flex-col gap-6 rounded-[20px] bg-white/[0.02] p-4 sm:p-5">
                <WalletCardIdentity
                    wallet={wallet}
                    isSelected={isSelected}
                    onSelect={onSelect}
                    copied={copied}
                    onCopy={handleCopy}
                    isEditing={isEditing}
                    labelInput={labelInput}
                    noteInput={noteInput}
                    onLabelChange={setLabelInput}
                    onNoteChange={setNoteInput}
                    onStartEditing={handleStartEditing}
                    onCancelEditing={() => setIsEditing(false)}
                    onSave={handleSaveMetadata}
                    isSaving={isSaving}
                />

                <WalletCardStats wallet={wallet} />

                <WalletCardActions
                    wallet={wallet}
                    onView={onView}
                    onEnable={onEnable}
                    onCopy={handleCopy}
                />
            </div>
        </div>
    );
}
