import { clsx } from 'clsx';
import * as React from "react";
import { twMerge } from 'tailwind-merge';

// Local cn function to avoid circular dependencies
function cn(...inputs: (string | undefined | null | false)[]) {
  return twMerge(clsx(inputs));
}

export interface TableProps extends React.HTMLAttributes<HTMLTableElement> {}

const Table = React.forwardRef<HTMLTableElement, TableProps>(
  ({ className, ...props }, ref) => (
    <div className="relative w-full overflow-auto">
      <table
        ref={ref}
        className={cn("w-full caption-bottom text-sm", className)}
        {...props}
      />
    </div>
  )
)
Table.displayName = "Table"

export interface TableHeaderProps extends React.HTMLAttributes<HTMLTableSectionElement> {}

const TableHeader = React.forwardRef<HTMLTableSectionElement, TableHeaderProps>(
  ({ className, ...props }, ref) => (
    <thead ref={ref} className={cn("[&_tr]:border-b [&_tr]:border-white/10", className)} {...props} />
  )
)
TableHeader.displayName = "Tableheader"

export interface TableBodyProps extends React.HTMLAttributes<HTMLTableSectionElement> {}

const TableBody = React.forwardRef<HTMLTableSectionElement, TableBodyProps>(
  ({ className, ...props }, ref) => (
    <tbody
      ref={ref}
      className={cn("[&_tr:last-child]:border-0", className)}
      {...props}
    />
  )
)
TableBody.displayName = "TableBody"

export interface TableFooterProps extends React.HTMLAttributes<HTMLTableSectionElement> {
  /** 
   * Styling variant for footer
   * - 'muted': Muted background (admin style)
   * - 'primary': Primary color background (frontend style)
   */
  variant?: 'muted' | 'primary';
}

const TableFooter = React.forwardRef<HTMLTableSectionElement, TableFooterProps>(
  ({ className, variant = 'muted', ...props }, ref) => (
    <tfoot
      ref={ref}
      className={cn(
        "font-medium [&>tr]:last:border-b-0",
        variant === 'muted' && "border-t border-white/10 bg-white/5 backdrop-blur-sm",
        variant === 'primary' && "bg-gradient-to-r from-purple-500 to-orange-500 text-white",
        className
      )}
      {...props}
    />
  )
)
TableFooter.displayName = "TableFooter"

export interface TableRowProps extends React.HTMLAttributes<HTMLTableRowElement> {}

const TableRow = React.forwardRef<HTMLTableRowElement, TableRowProps>(
  ({ className, ...props }, ref) => (
    <tr
      ref={ref}
      className={cn(
        "border-b border-white/5 transition-colors hover:bg-white/5 data-[state=selected]:bg-white/10",
        className
      )}
      {...props}
    />
  )
)
TableRow.displayName = "TableRow"

export interface TableHeadProps extends React.ThHTMLAttributes<HTMLTableCellElement> {}

const TableHead = React.forwardRef<HTMLTableCellElement, TableHeadProps>(
  ({ className, ...props }, ref) => (
    <th
      ref={ref}
      className={cn(
        "h-12 px-4 text-left align-middle font-medium text-muted-foreground [&:has([role=checkbox])]:pr-0",
        className
      )}
      {...props}
    />
  )
)
TableHead.displayName = "TableHead"

export interface TableCellProps extends React.TdHTMLAttributes<HTMLTableCellElement> {}

const TableCell = React.forwardRef<HTMLTableCellElement, TableCellProps>(
  ({ className, ...props }, ref) => (
    <td
      ref={ref}
      className={cn("p-4 align-middle [&:has([role=checkbox])]:pr-0", className)}
      {...props}
    />
  )
)
TableCell.displayName = "TableCell"

export interface TableCaptionProps extends React.HTMLAttributes<HTMLTableCaptionElement> {}

const TableCaption = React.forwardRef<HTMLTableCaptionElement, TableCaptionProps>(
  ({ className, ...props }, ref) => (
    <caption
      ref={ref}
      className={cn("mt-4 text-sm text-muted-foreground", className)}
      {...props}
    />
  )
)
TableCaption.displayName = "TableCaption"

export {
    Table, TableBody, TableCaption, TableCell, TableFooter,
    TableHead, TableHeader, TableRow
};

