// Type overrides to fix React types compatibility issues
declare module '@epsx/ui' {
  import { FC, ReactNode } from 'react';
  
  interface CardProps {
    children?: ReactNode;
    className?: string;
  }
  
  interface CardHeaderProps {
    children?: ReactNode;
    className?: string;
  }
  
  interface CardTitleProps {
    children?: ReactNode;
    className?: string;
  }
  
  interface CardContentProps {
    children?: ReactNode;
    className?: string;
  }
  
  export const Card: FC<CardProps>;
  export const CardHeader: FC<CardHeaderProps>;
  export const CardTitle: FC<CardTitleProps>;
  export const CardContent: FC<CardContentProps>;
}