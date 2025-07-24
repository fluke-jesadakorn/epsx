'use client';

import { ReactNode } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ChevronRight, MoreVertical } from 'lucide-react';

interface MobileCardProps {
  title?: string;
  subtitle?: string;
  children: ReactNode;
  action?: {
    label: string;
    onClick: () => void;
  };
  showMoreButton?: boolean;
  onMoreClick?: () => void;
  className?: string;
  padding?: 'none' | 'sm' | 'md' | 'lg';
  clickable?: boolean;
  onClick?: () => void;
}

export function MobileCard({
  title,
  subtitle,
  children,
  action,
  showMoreButton = false,
  onMoreClick,
  className = '',
  padding = 'md',
  clickable = false,
  onClick
}: MobileCardProps) {
  const paddingClasses = {
    none: 'p-0',
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-6'
  };

  const cardContent = (
    <>
      {(title || subtitle || action || showMoreButton) && (
        <CardHeader className={`${paddingClasses[padding]} pb-3`}>
          <div className="flex items-center justify-between">
            <div className="flex-1 min-w-0">
              {title && (
                <CardTitle className="text-base font-semibold truncate">
                  {title}
                </CardTitle>
              )}
              {subtitle && (
                <p className="text-sm text-muted-foreground mt-1 truncate">
                  {subtitle}
                </p>
              )}
            </div>
            
            <div className="flex items-center gap-2 ml-3">
              {action && (
                <Button 
                  variant="ghost" 
                  size="sm"
                  onClick={action.onClick}
                  className="text-primary hover:text-primary/80"
                >
                  {action.label}
                  <ChevronRight className="h-4 w-4 ml-1" />
                </Button>
              )}
              
              {showMoreButton && (
                <Button 
                  variant="ghost" 
                  size="sm"
                  onClick={onMoreClick}
                  className="p-1"
                >
                  <MoreVertical className="h-4 w-4" />
                </Button>
              )}
            </div>
          </div>
        </CardHeader>
      )}
      
      <CardContent className={`${paddingClasses[padding]} ${title || subtitle ? 'pt-0' : ''}`}>
        {children}
      </CardContent>
    </>
  );

  if (clickable && onClick) {
    return (
      <Card 
        className={`${className} cursor-pointer transition-colors hover:bg-muted/30 active:bg-muted/50`}
        onClick={onClick}
      >
        {cardContent}
      </Card>
    );
  }

  return (
    <Card className={className}>
      {cardContent}
    </Card>
  );
}