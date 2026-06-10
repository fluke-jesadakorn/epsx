/**
 * FRONTEND UI COMPONENTS INDEX
 * Re-exports from shared components where possible
 * Keeps app-specific components local
 */

// ============================================================================
// BASIC PRIMITIVES - Re-export from shared
// ============================================================================
export { Alert, AlertDescription, type AlertProps, type AlertDescriptionProps } from '@/shared/components/ui/alert';
export { Badge, badgeVariants, type BadgeProps } from '@/shared/components/ui/badge';
export { Button, buttonVariants, type ButtonProps } from '@/shared/components/ui/button';
export { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/shared/components/ui/card';
export { Input, inputVariants, type InputProps } from '@/shared/components/ui/input';
export { Label } from '@/shared/components/ui/label';
export { Skeleton, type SkeletonProps } from '@/shared/components/ui/skeleton';
export { Switch } from '@/shared/components/ui/switch';
export { Tabs, TabsContent, TabsList, TabsTrigger, type TabsContentProps, type TabsListProps, type TabsTriggerProps } from '@/shared/components/ui/tabs';
export { Textarea } from '@/shared/components/ui/textarea';
export { Progress, type ProgressProps } from '@/shared/components/ui/progress';
export { Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow } from '@/shared/components/ui/table';
export { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogOverlay, DialogPortal, DialogTitle, DialogTrigger } from '@/shared/components/ui/dialog';
export { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/components/ui/tooltip';
export { Avatar } from '@/shared/components/ui/avatar';
export { Popover, PopoverContent, PopoverTrigger } from '@/shared/components/ui/popover';
export { ScrollArea, ScrollBar } from '@/shared/components/ui/scroll-area';
export { Toast, ToastAction, ToastClose, ToastDescription, ToastProvider, ToastTitle, ToastViewport, type ToastProps } from '@/shared/components/ui/toast';
export { Toaster } from '@/shared/components/ui/toaster';
export { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from '@/shared/components/ui/alert-dialog';

// ============================================================================
// NAVIGATION AND OVERLAY - Re-export from shared
// ============================================================================
export {
  DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuPortal, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuSeparator,
  DropdownMenuShortcut, DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger, DropdownMenuTrigger
} from '@/shared/components/ui/dropdown-menu';
export {
  Sheet, SheetClose,
  SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetOverlay, SheetPortal, SheetTitle, SheetTrigger
} from '@/shared/components/ui/sheet';
export {
  Select,
  SelectContent, SelectGroup, SelectItem, SelectLabel, SelectScrollDownButton, SelectScrollUpButton, SelectSeparator, SelectTrigger,
  SelectValue
} from '@/shared/components/ui/select';

// ============================================================================
// SHARED COMPONENTS
// ============================================================================
export {
  UnifiedLoader, UnifiedLoading, UnifiedProgressBar, UnifiedSkeleton,
  type UnifiedLoaderProps, type UnifiedLoadingProps, type UnifiedProgressBarProps, type UnifiedSkeletonProps
} from '@/shared/components/loaders/loader';

export {
  AnimatedThemeToggle, GradientThemeToggle,
  MinimalThemeToggle, OptimizedThemeToggle, ThemeToggle,
  ThemeToggleCSS, UnifiedThemeToggle, type ThemeToggleIconType,
  type ThemeToggleSize, type ThemeToggleVariant, type UnifiedThemeToggleProps
} from '@/shared/components/ui/theme-toggle';

export {
  MetroNotification, ProfessionalAlert, ProfessionalNotification, UnifiedAlert, UnifiedNotification, useAdminToast,
  useAnalyticsToast, useMetroToast, usePancakeToast, useProfessionalToast, useUnifiedToast,
  type UnifiedNotificationProps, type UnifiedAlertProps, type ToastNotification, type UseUnifiedToastProps
} from '@/shared/components/notifications/notification';

// ============================================================================
// FRONTEND-SPECIFIC COMPONENTS - Keep local
// ============================================================================
export { Collapsible, CollapsibleContent, CollapsibleTrigger } from './collapsible';
export { Separator } from './separator';
export { Checkbox } from './checkbox';
export { InputWithIcon } from './input-with-icon';
export { LoadingButton } from './loading-button';
export { PermissionBadge } from './permission-badge';

// ============================================================================
// FORM COMPONENTS - Keep local wrapper for React Hook Form integration
// ============================================================================
export {
  Form, FormControl,
  FormDescription, FormField,
  FormItem,
  FormLabel, FormMessage,
  useFormField
} from './form';
