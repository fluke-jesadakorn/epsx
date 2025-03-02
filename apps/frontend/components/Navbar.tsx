"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useAuthState } from "react-firebase-hooks/auth";
import { auth } from "@/lib/firebase-client";
import { signOut } from "firebase/auth";
import { LineChart, User, LogOut, LogIn, File } from "lucide-react";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import ThemeToggle from "./ThemeToggle";

const getNavItems = (isLoggedIn: boolean) => [
  {
    label: "Docs",
    href: "https://your-gitbook-url.com",
    key: "docs",
    icon: <File className="h-4 w-4" />,
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
    <div className="bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 border-b backdrop-blur-sm">
      <div className="flex h-20 items-center px-6 justify-between max-w-7xl mx-auto">
        <div className="flex items-center gap-8">
          <Link href="/" className="flex items-center gap-2 font-bold text-xl">
            <span className="bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
              EPSX
            </span>
          </Link>
          <NavigationMenu>
            <NavigationMenuList>
              {getNavItems(isLoggedIn).map((item) => (
                <NavigationMenuItem key={item.key}>
                  <Link href={item.href} legacyBehavior passHref>
                    <NavigationMenuLink
                      className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium transition-all rounded-full hover:bg-primary/10
                      ${
                        pathname === item.href
                          ? "bg-primary/10 text-primary"
                          : "text-muted-foreground hover:text-primary"
                      }`}
                    >
                      {item.icon}
                      {item.label}
                    </NavigationMenuLink>
                  </Link>
                </NavigationMenuItem>
              ))}
            </NavigationMenuList>
          </NavigationMenu>
        </div>

        <div className="flex items-center gap-6">
          <ThemeToggle />

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
              className="flex items-center gap-2 rounded-full hover:bg-primary/10"
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
