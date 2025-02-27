"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTheme } from "next-themes";
import { useAuthState } from "react-firebase-hooks/auth";
import { auth } from "@/lib/firebase-client";
import { signOut } from "firebase/auth";
import {
  Home,
  LineChart,
  User,
  LogOut,
  LogIn,
  Sun,
  Moon,
} from "lucide-react";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

const getNavItems = (isLoggedIn: boolean) => [
  {
    label: "Home",
    href: "/",
    key: "home",
    icon: <Home className="h-4 w-4" />,
  },
  {
    label: "Ranking",
    href: "/ranking",
    key: "ranking",
    icon: <LineChart className="h-4 w-4" />,
  },
  ...(isLoggedIn
    ? [
        {
          label: "Settings",
          href: "/settings",
          key: "settings",
          icon: <User className="h-4 w-4" />,
        },
      ]
    : []),
];

export default function Navbar() {
  const { theme, setTheme } = useTheme();
  const pathname = usePathname();
  const [user] = useAuthState(auth);
  const isLoggedIn = !!user;
  const userEmail = user?.email;

  const logout = async () => {
    try {
      await signOut(auth);
    } catch (error) {
      console.error("Error signing out:", error);
    }
  };

  return (
    <div className="border-b">
      <div className="flex h-16 items-center px-4 justify-between max-w-7xl mx-auto">
        <NavigationMenu>
          <NavigationMenuList>
            {getNavItems(isLoggedIn).map((item) => (
              <NavigationMenuItem key={item.key}>
                <Link href={item.href} legacyBehavior passHref>
                  <NavigationMenuLink
                    className={`flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors hover:text-primary
                      ${pathname === item.href ? "text-primary" : "text-muted-foreground"}`}
                  >
                    {item.icon}
                    {item.label}
                  </NavigationMenuLink>
                </Link>
              </NavigationMenuItem>
            ))}
          </NavigationMenuList>
        </NavigationMenu>
        
        <div className="flex items-center gap-4">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <div className="flex items-center gap-2">
                  <Switch
                    checked={theme === "dark"}
                    onCheckedChange={(checked) => setTheme(checked ? "dark" : "light")}
                    className="data-[state=checked]:bg-slate-700"
                  />
                  {theme === "dark" ? (
                    <Moon className="h-4 w-4 text-slate-400" />
                  ) : (
                    <Sun className="h-4 w-4 text-yellow-500" />
                  )}
                </div>
              </TooltipTrigger>
              <TooltipContent>
                <p>{theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
          
          {isLoggedIn && userEmail && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Link href="/settings">
                    <Avatar className="h-8 w-8">
                      <AvatarFallback>
                        {userEmail[0].toUpperCase()}
                      </AvatarFallback>
                    </Avatar>
                  </Link>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{userEmail}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
          
          <Link href={isLoggedIn ? "#" : "/login"}>
            <Button
              variant="ghost"
              onClick={isLoggedIn ? logout : undefined}
              className="flex items-center gap-2"
            >
              {isLoggedIn ? (
                <>
                  <LogOut className="h-4 w-4" /> Logout
                </>
              ) : (
                <>
                  <LogIn className="h-4 w-4" /> Login
                </>
              )}
            </Button>
          </Link>
        </div>
      </div>
    </div>
  );
}
