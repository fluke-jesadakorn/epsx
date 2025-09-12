/**
 * UI & Component Types
 * Consolidates all UI-related types and component interfaces
 * Replaces scattered UI types across component files
 */

import type { ReactNode } from 'react';
import type { User, Permission, Notification } from './core';

// ============================================================================
// Generic UI Component Types
// ============================================================================

export interface BaseComponentProps {
  className?: string;
  children?: ReactNode;
  id?: string;
  'data-testid'?: string;
}

export interface LoadingState {
  isLoading: boolean;
  error?: string | null;
  message?: string;
}

export interface AsyncState<T> extends LoadingState {
  data?: T | null;
}

export type ComponentSize = 'sm' | 'md' | 'lg' | 'xl';
export type ComponentVariant = 'default' | 'primary' | 'secondary' | 'success' | 'warning' | 'error';
export type ComponentState = 'default' | 'hover' | 'active' | 'disabled' | 'loading';

// ============================================================================
// Form Types
// ============================================================================

export interface FormProps<T = any> extends BaseComponentProps {
  onSubmit: (data: T) => void | Promise<void>;
  onCancel?: () => void;
  isLoading?: boolean;
  initialData?: Partial<T>;
  validationMode?: 'onChange' | 'onBlur' | 'onSubmit';
}

export interface FieldProps<T = any> {
  name: string;
  label?: string;
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  error?: string;
  helperText?: string;
  value?: T;
  onChange?: (value: T) => void;
  validation?: {
    required?: boolean;
    minLength?: number;
    maxLength?: number;
    pattern?: RegExp;
    validator?: (value: T) => string | null;
  };
}

export interface SelectOption<T = string> {
  label: string;
  value: T;
  disabled?: boolean;
  icon?: ReactNode;
  description?: string;
}

export interface MultiSelectProps<T = string> {
  options: SelectOption<T>[];
  value: T[];
  onChange: (value: T[]) => void;
  placeholder?: string;
  searchable?: boolean;
  maxItems?: number;
  disabled?: boolean;
}

// ============================================================================
// Table Types
// ============================================================================

export interface TableColumn<T = any> {
  key: string;
  label: string;
  sortable?: boolean;
  searchable?: boolean;
  width?: string | number;
  minWidth?: string | number;
  align?: 'left' | 'center' | 'right';
  render?: (value: any, item: T, index: number) => ReactNode;
  className?: string;
}

export interface TableProps<T = any> {
  columns: TableColumn<T>[];
  data: T[];
  loading?: boolean;
  error?: string;
  emptyMessage?: string;
  selectable?: boolean;
  selectedItems?: T[];
  onSelectionChange?: (items: T[]) => void;
  onItemClick?: (item: T) => void;
  pagination?: PaginationProps;
  sortable?: boolean;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  onSort?: (column: string, order: 'asc' | 'desc') => void;
  className?: string;
}

export interface PaginationProps {
  currentPage: number;
  totalPages: number;
  totalItems: number;
  itemsPerPage: number;
  onPageChange: (page: number) => void;
  onItemsPerPageChange?: (itemsPerPage: number) => void;
  showSizeChanger?: boolean;
  showQuickJumper?: boolean;
  size?: ComponentSize;
}

// ============================================================================
// Modal & Dialog Types
// ============================================================================

export interface ModalProps extends BaseComponentProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  description?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  closable?: boolean;
  maskClosable?: boolean;
  footer?: ReactNode;
  showCloseButton?: boolean;
}

export interface ConfirmDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void | Promise<void>;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'info' | 'warning' | 'error' | 'success';
  loading?: boolean;
}

export interface AlertProps extends BaseComponentProps {
  type: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  message: string;
  closable?: boolean;
  onClose?: () => void;
  actions?: ReactNode;
}

// ============================================================================
// Navigation & Menu Types
// ============================================================================

export interface NavItem {
  id: string;
  label: string;
  icon?: ReactNode;
  href?: string;
  onClick?: () => void;
  children?: NavItem[];
  badge?: string | number;
  disabled?: boolean;
  hidden?: boolean;
  requiredPermission?: string;
}

export interface BreadcrumbItem {
  label: string;
  href?: string;
  icon?: ReactNode;
}

export interface TabItem {
  id: string;
  label: string;
  content: ReactNode;
  icon?: ReactNode;
  disabled?: boolean;
  badge?: string | number;
  requiredPermission?: string;
}

export interface TabsProps {
  items: TabItem[];
  activeTab: string;
  onTabChange: (tabId: string) => void;
  variant?: 'line' | 'card' | 'pill';
  size?: ComponentSize;
  className?: string;
}

// ============================================================================
// Layout Types
// ============================================================================

export interface LayoutProps {
  children: ReactNode;
  sidebar?: ReactNode;
  header?: ReactNode;
  footer?: ReactNode;
  breadcrumbs?: BreadcrumbItem[];
  title?: string;
  description?: string;
  actions?: ReactNode;
  className?: string;
}

export interface SidebarProps {
  collapsed?: boolean;
  onToggle?: () => void;
  items: NavItem[];
  currentPath: string;
  className?: string;
}

export interface HeaderProps {
  title?: string;
  breadcrumbs?: BreadcrumbItem[];
  actions?: ReactNode;
  user?: User;
  onLogout?: () => void;
  className?: string;
}

// ============================================================================
// Card & Panel Types
// ============================================================================

export interface CardProps extends BaseComponentProps {
  title?: string;
  description?: string;
  header?: ReactNode;
  footer?: ReactNode;
  actions?: ReactNode;
  loading?: boolean;
  hoverable?: boolean;
  bordered?: boolean;
  size?: ComponentSize;
}

export interface StatsCardProps {
  title: string;
  value: string | number;
  change?: {
    value: string | number;
    type: 'increase' | 'decrease';
    period?: string;
  };
  icon?: ReactNode;
  loading?: boolean;
  className?: string;
}

export interface MetricCardProps {
  label: string;
  value: string | number;
  unit?: string;
  trend?: {
    direction: 'up' | 'down' | 'neutral';
    value: string | number;
    isGood?: boolean;
  };
  color?: string;
  icon?: ReactNode;
}

// ============================================================================
// Data Visualization Types
// ============================================================================

export interface ChartData {
  labels: string[];
  datasets: Array<{
    label: string;
    data: number[];
    backgroundColor?: string | string[];
    borderColor?: string;
    borderWidth?: number;
  }>;
}

export interface ChartOptions {
  responsive?: boolean;
  maintainAspectRatio?: boolean;
  plugins?: {
    legend?: {
      display?: boolean;
      position?: 'top' | 'bottom' | 'left' | 'right';
    };
    tooltip?: {
      enabled?: boolean;
    };
  };
  scales?: {
    x?: {
      display?: boolean;
    };
    y?: {
      display?: boolean;
      beginAtZero?: boolean;
    };
  };
}

export interface ChartProps {
  data: ChartData;
  options?: ChartOptions;
  type: 'line' | 'bar' | 'pie' | 'doughnut' | 'area';
  width?: number;
  height?: number;
  className?: string;
}

// ============================================================================
// User Management UI Types
// ============================================================================

export interface UserCardProps {
  user: User;
  showActions?: boolean;
  onEdit?: (user: User) => void;
  onDelete?: (user: User) => void;
  onViewPermissions?: (user: User) => void;
  className?: string;
}

export interface UserFormData {
  email: string;
  firstName?: string;
  lastName?: string;
  role: string;
  packageTier: string;
  permissions: string[];
  isActive: boolean;
}

export interface UserFormProps extends FormProps<UserFormData> {
  availableRoles: SelectOption[];
  availablePermissions: SelectOption[];
  availablePackageTiers: SelectOption[];
  mode: 'create' | 'edit';
}

export interface UserListProps {
  users: User[];
  loading?: boolean;
  error?: string;
  onUserSelect?: (user: User) => void;
  onUserEdit?: (user: User) => void;
  onUserDelete?: (user: User) => void;
  onBulkAction?: (action: string, users: User[]) => void;
  selectedUsers?: User[];
  onSelectionChange?: (users: User[]) => void;
  showBulkActions?: boolean;
  className?: string;
}

// ============================================================================
// Permission Management UI Types
// ============================================================================

export interface PermissionTileProps {
  permission: Permission;
  granted: boolean;
  onToggle: (permission: Permission, granted: boolean) => void;
  disabled?: boolean;
  showExpiry?: boolean;
  className?: string;
}

export interface PermissionFormData {
  userId: string;
  permissions: string[];
  expiresAt?: string;
  reason?: string;
}

export interface PermissionFormProps extends FormProps<PermissionFormData> {
  availablePermissions: SelectOption[];
  users: User[];
  mode: 'grant' | 'revoke' | 'bulk';
}

export interface PermissionTableProps {
  permissions: Permission[];
  users?: User[];
  onPermissionRevoke?: (permission: Permission) => void;
  onPermissionExtend?: (permission: Permission, newExpiryDate: string) => void;
  showUserInfo?: boolean;
  groupByUser?: boolean;
  className?: string;
}

// ============================================================================
// Notification UI Types
// ============================================================================

export interface NotificationItemProps {
  notification: Notification;
  onRead?: (id: string) => void;
  onDelete?: (id: string) => void;
  onAction?: (notification: Notification) => void;
  className?: string;
}

export interface NotificationListProps {
  notifications: Notification[];
  loading?: boolean;
  onNotificationRead?: (id: string) => void;
  onNotificationDelete?: (id: string) => void;
  onMarkAllRead?: () => void;
  showActions?: boolean;
  className?: string;
}

export interface NotificationFormData {
  title: string;
  message: string;
  type: Notification['type'];
  priority: Notification['priority'];
  recipients: 'all' | 'specific';
  userIds?: string[];
  actionUrl?: string;
}

export interface NotificationFormProps extends FormProps<NotificationFormData> {
  users: User[];
  mode: 'create' | 'edit';
}

// ============================================================================
// Search & Filter Types
// ============================================================================

export interface SearchProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  onSearch?: (value: string) => void;
  loading?: boolean;
  className?: string;
}

export interface FilterProps<T = any> {
  filters: T;
  onFiltersChange: (filters: T) => void;
  onReset?: () => void;
  className?: string;
}

export interface QuickFilterItem<T = any> {
  label: string;
  value: T;
  count?: number;
}

export interface QuickFiltersProps<T = any> {
  items: QuickFilterItem<T>[];
  value: T;
  onChange: (value: T) => void;
  className?: string;
}

// ============================================================================
// Theme & Styling Types
// ============================================================================

export interface ThemeColors {
  primary: string;
  secondary: string;
  success: string;
  warning: string;
  error: string;
  info: string;
  background: string;
  surface: string;
  text: {
    primary: string;
    secondary: string;
    disabled: string;
  };
}

export interface ThemeConfig {
  colors: ThemeColors;
  spacing: {
    xs: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
  };
  borderRadius: {
    sm: string;
    md: string;
    lg: string;
  };
  shadows: {
    sm: string;
    md: string;
    lg: string;
  };
}

// ============================================================================
// Toast & Notification System Types
// ============================================================================

export interface ToastProps {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title?: string;
  message: string;
  duration?: number;
  actions?: Array<{
    label: string;
    onClick: () => void;
  }>;
}

export interface ToastContextValue {
  toasts: ToastProps[];
  showToast: (toast: Omit<ToastProps, 'id'>) => void;
  removeToast: (id: string) => void;
  clearAll: () => void;
}

// ============================================================================
// Utility Component Types
// ============================================================================

export interface CopyButtonProps {
  text: string;
  successMessage?: string;
  size?: ComponentSize;
  variant?: ComponentVariant;
  className?: string;
}

export interface EmptyStateProps {
  title: string;
  description?: string;
  icon?: ReactNode;
  actions?: ReactNode;
  className?: string;
}

export interface LoadingSpinnerProps {
  size?: ComponentSize;
  color?: string;
  className?: string;
}

export interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  count?: number;
  className?: string;
}

// ============================================================================
// Event Handler Types
// ============================================================================

export type MouseEventHandler = (event: React.MouseEvent) => void;
export type ChangeEventHandler<T = string> = (value: T) => void;
export type SubmitEventHandler<T = any> = (data: T) => void | Promise<void>;
export type SelectEventHandler<T = any> = (item: T) => void;

// Generic event handlers for common operations
export interface CRUDEventHandlers<T = any> {
  onCreate?: (item: Partial<T>) => void | Promise<void>;
  onRead?: (id: string) => void | Promise<void>;
  onUpdate?: (id: string, data: Partial<T>) => void | Promise<void>;
  onDelete?: (id: string) => void | Promise<void>;
  onBulkDelete?: (ids: string[]) => void | Promise<void>;
}