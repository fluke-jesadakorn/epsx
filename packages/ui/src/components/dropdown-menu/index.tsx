"use client";

import * as React from "react";

import { cn } from "../../lib/utils";

interface DropdownMenuProps {
  children: React.ReactNode;
}

interface DropdownMenuTriggerProps {
  asChild?: boolean;
  children: React.ReactNode;
}

interface DropdownMenuContentProps {
  align?: "start" | "center" | "end";
  children: React.ReactNode;
}

interface DropdownMenuItemProps {
  onClick?: () => void;
  children: React.ReactNode;
}

const DropdownMenuContext = React.createContext<{
  isOpen: boolean;
  setIsOpen: (open: boolean) => void;
} | null>(null);

const DropdownMenu: React.FC<DropdownMenuProps> = ({ children }) => {
  const [isOpen, setIsOpen] = React.useState(false);

  return (
    <DropdownMenuContext.Provider value={{ isOpen, setIsOpen }}>
      <div className="relative inline-block text-left">{children}</div>
    </DropdownMenuContext.Provider>
  );
};

const DropdownMenuTrigger: React.FC<DropdownMenuTriggerProps> = ({ 
  asChild, 
  children 
}) => {
  const context = React.useContext(DropdownMenuContext);
  if (!context) throw new Error("DropdownMenuTrigger must be used within DropdownMenu");

  const handleClick = (): void => {
    context.setIsOpen(!context.isOpen);
  };

  if (asChild) {
    return (
      <div onClick={handleClick}>
        {children}
      </div>
    );
  }

  return (
    <button onClick={handleClick}>
      {children}
    </button>
  );
};

const DropdownMenuContent: React.FC<DropdownMenuContentProps> = ({ 
  align = "end", 
  children 
}) => {
  const context = React.useContext(DropdownMenuContext);
  if (!context) throw new Error("DropdownMenuContent must be used within DropdownMenu");

  React.useEffect(() => {
    const handleClickOutside = (e: MouseEvent): void => {
      const target = e.target as Element;
      if (!target.closest("[data-dropdown-menu]")) {
        context.setIsOpen(false);
      }
    };

    if (context.isOpen) {
      document.addEventListener("click", handleClickOutside);
    }

    return (): void => {
      document.removeEventListener("click", handleClickOutside);
    };
  }, [context.isOpen, context]);

  if (!context.isOpen) return null;

  return (
    <div
      data-dropdown-menu
      className={cn(
        "absolute z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md",
        {
          "right-0": align === "end",
          "left-0": align === "start",
          "left-1/2 transform -translate-x-1/2": align === "center",
        }
      )}
    >
      {children}
    </div>
  );
};

const DropdownMenuItem: React.FC<DropdownMenuItemProps> = ({ onClick, children }) => {
  const context = React.useContext(DropdownMenuContext);
  if (!context) throw new Error("DropdownMenuItem must be used within DropdownMenu");

  const handleClick = (): void => {
    onClick?.();
    context.setIsOpen(false);
  };

  return (
    <div
      onClick={handleClick}
      className="relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none hover:bg-accent hover:text-accent-foreground"
    >
      {children}
    </div>
  );
};

export { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem };
