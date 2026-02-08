/**
 * SHARED UI PRIMITIVES
 * Consolidated from admin-frontend and frontend apps
 * These are base Radix/Tailwind components used across all applications
 */

// Utility
export { cn } from '../../utils/cn';

// Basic Elements
export { Badge, badgeVariants, type BadgeProps } from './badge';
export { Button, buttonVariants, type ButtonProps } from './button';
export { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from './card';
export { Input, inputVariants, type InputProps } from './input';

// Dropdown Menu
export {
    DropdownMenu,
    DropdownMenuCheckboxItem,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuPortal,
    DropdownMenuRadioGroup,
    DropdownMenuRadioItem,
    DropdownMenuSeparator,
    DropdownMenuShortcut,
    DropdownMenuSub,
    DropdownMenuSubContent,
    DropdownMenuSubTrigger,
    DropdownMenuTrigger
} from './dropdown-menu';

// Alert
export {
    Alert, AlertDescription, AlertTitle, alertVariants, type AlertDescriptionProps, type AlertProps,
    type AlertTitleProps
} from './alert';

// Dialog
export {
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogOverlay, DialogPortal, DialogTitle, DialogTrigger, type DialogContentProps, type DialogOverlayProps
} from './dialog';

// Table
export {
    Table, TableBody, TableCaption, TableCell, TableFooter,
    TableHead, TableHeader, TableRow, type TableBodyProps, type TableCaptionProps, type TableCellProps, type TableFooterProps, type TableHeadProps, type TableHeaderProps, type TableProps, type TableRowProps
} from './table';

// Skeleton
export { Skeleton, type SkeletonProps } from './skeleton';

// Progress
export { Progress, type ProgressProps } from './progress';

// Tabs
export {
    Tabs, TabsContent, TabsList,
    TabsTrigger, type TabsContentProps, type TabsListProps,
    type TabsTriggerProps
} from './tabs';

// Theme Toggle
export {
    AdminThemeToggle, AnimatedThemeToggle, GradientThemeToggle,
    MinimalThemeToggle, OptimizedThemeToggle, SimpleThemeToggle, ThemeToggle,
    ThemeToggleCSS, UnifiedThemeToggle, type ThemeToggleIconType,
    type ThemeToggleSize, type ThemeToggleVariant, type UnifiedThemeToggleProps
} from './unified-theme-toggle';

// Tooltip
export {
    Tooltip, TooltipContent, TooltipProvider, TooltipTrigger
} from './tooltip';

// Safe Theme Script (for preventing FOUC)
export {
    SafeThemeScript,
    SafeThemeScriptWithNonce,
    themeUtils,
    type ValidTheme
} from './safe-theme-script';

// Transfer List
export { TransferList, type TransferListProps } from './transfer-list';
