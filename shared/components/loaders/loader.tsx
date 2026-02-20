'use client';

import type { ReactNode } from 'react';
import { cn } from '../../utils/cn';

import {
    BarsLoader,
    DotsLoader,
    loaderVariants,
    PulseLoader,
    SpinnerLoader,
    StackLoader,
    type LoaderSize,
    type LoaderVariant
} from './loader-variants';

type LoaderType = 'spinner' | 'dots' | 'bars' | 'pulse' | 'stack';

export interface UnifiedLoaderProps {
    variant?: LoaderVariant;
    size?: LoaderSize;
    type?: LoaderType;
    message?: string;
    children?: ReactNode;
    className?: string;
}

export function UnifiedLoader({
    variant = 'default',
    size = 'md',
    type = 'spinner',
    message,
    children,
    className
}: UnifiedLoaderProps) {
    const style = loaderVariants[variant]

    const renderLoader = () => {
        switch (type) {
            case 'dots':
                return <DotsLoader size={size} style={style} />
            case 'bars':
                return <BarsLoader size={size} style={style} />
            case 'pulse':
                return <PulseLoader size={size} style={style} />
            case 'stack':
                return <StackLoader size={size} style={style} />
            default:
                return <SpinnerLoader size={size} style={style} />
        }
    }

    return (
        <div className={cn('flex flex-col items-center justify-center gap-3', className)}>
            {renderLoader()}

            {/* Progress Dots for stack type */}
            {type === 'stack' && (
                <div className="flex space-x-2">
                    {Array.from({ length: 3 }).map((_, i) => (
                        <div
                            // eslint-disable-next-line react/no-array-index-key
                            key={`stack-dot-${i}`}
                            className={cn('w-2 h-2 rounded-none', style.accent)}
                        />
                    ))}
                </div>
            )}

            {message !== undefined && message !== '' && (
                <p className={cn('text-sm font-medium animate-pulse', style.text)}>
                    {message}
                </p>
            )}

            {children}
        </div>
    )
}

export * from './loading';
export * from './progress-bar';
export * from './skeleton';

