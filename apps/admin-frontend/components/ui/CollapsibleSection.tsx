'use client';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { ChevronDown, ChevronUp, FileText, Layout, Shield } from 'lucide-react';
import { useState } from 'react';

interface CollapsibleSectionProps {
    title: string;
    children: React.ReactNode;
    defaultOpen?: boolean;
    icon?: 'FileText' | 'Layout' | 'Shield';
    className?: string;
}

const Icons = {
    FileText,
    Layout,
    Shield
};

export function CollapsibleSection({
    title,
    children,
    defaultOpen = false,
    icon,
    className
}: CollapsibleSectionProps) {
    const [isOpen, setIsOpen] = useState(defaultOpen);
    const IconComponent = icon ? Icons[icon] : null;

    return (
        <div className={cn("bg-card border border-border rounded-xl shadow-sm overflow-hidden", className)}>
            <div
                className="flex items-center justify-between p-4 cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => setIsOpen(!isOpen)}
            >
                <div className="flex items-center gap-2">
                    {IconComponent && <IconComponent className="h-5 w-5 text-muted-foreground" />}
                    <h3 className="font-semibold text-lg">{title}</h3>
                </div>
                <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                    {isOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                </Button>
            </div>

            {isOpen && (
                <div className="p-4 pt-0 border-t border-border/50 animate-in slide-in-from-top-2 duration-200">
                    <div className="mt-4">
                        {children}
                    </div>
                </div>
            )}
        </div>
    );
}
