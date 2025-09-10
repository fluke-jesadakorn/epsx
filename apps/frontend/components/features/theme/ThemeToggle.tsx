"use client";

import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";

import { Button } from "@/components/ui";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { useNavbarContext } from "@/components/providers/NavbarProvider";

export default function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const { isHydrated } = useNavbarContext();

  if (!isHydrated) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              disabled
              className="h-12 w-12 bg-gradient-to-r from-purple-400 to-pink-500 text-white rounded-2xl font-semibold shadow-lg flex items-center justify-center opacity-50 disabled:pointer-events-none"
            >
              <span className="text-xl">🌙</span>
              <span className="sr-only">Toggle theme</span>
            </button>
          </TooltipTrigger>
          <TooltipContent>
            <p>Toggle theme</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <button
            onClick={() => setTheme(theme === "light" ? "dark" : "light")}
            className="h-12 w-12 bg-gradient-to-r from-purple-400 to-pink-500 text-white rounded-2xl font-semibold hover:from-purple-500 hover:to-pink-600 shadow-lg hover:shadow-xl flex items-center justify-center"
          >
            <span className="text-xl">
              {theme === "light" ? "🌙" : "☀️"}
            </span>
            <span className="sr-only">Toggle theme</span>
          </button>
        </TooltipTrigger>
        <TooltipContent>
          <p>Toggle theme</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
