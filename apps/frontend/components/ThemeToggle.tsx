"use client";

import { useTheme } from "next-themes";
import { useEffect, useState } from "react";
import { Moon, Sun } from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export default function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return null;
  }

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <button
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            className="relative w-16 h-8 rounded-full bg-gray-200 dark:bg-gray-600 transition-colors"
          >
            <div
              className={`absolute top-1 left-1 w-6 h-6 rounded-full bg-white shadow-md transition-transform duration-300 transform ${
                theme === "dark" ? "translate-x-8" : "translate-x-0"
              } flex items-center justify-center`}
            >
              {theme === "dark" ? (
                <Moon className="h-4 w-4 text-gray-600" />
              ) : (
                <Sun className="h-4 w-4 text-amber-500" />
              )}
            </div>
          </button>
        </TooltipTrigger>
        <TooltipContent>
          <p>
            {theme === "dark"
              ? "Switch to light mode"
              : "Switch to dark mode"}
          </p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
