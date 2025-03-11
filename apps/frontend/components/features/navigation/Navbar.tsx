"use client";

import { useState, memo } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { LineChart, User, LogOut, LogIn, File, Menu, Settings } from "lucide-react";
import { useAuth } from "@/context/auth-context";
import { navigationService } from "@/services/navigation.service";
import { UserRole } from "@/types/auth/roles";
import { authService } from "@/services/auth.service";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import ThemeToggle from "@/components/features/theme/ThemeToggle";

const iconMap = {
  docs: <File className="h-4 w-4" />,
  ranking: <LineChart className="h-4 w-4" />,
  settings: <Settings className="h-4 w-4" />,
  admin: <Settings className="h-4 w-4" />,
};

function NavbarComponent() {
  const pathname = usePathname();
  const { isLoggedIn, userEmail, isAdmin, checkStatus } = useAuth();
  const [isOpen, setIsOpen] = useState(false);
  const router = useRouter();

  const navItems = navigationService.getNavItems(isLoggedIn, isAdmin ? UserRole.ADMINISTRATOR : undefined);

  const handleLogout = async () => {
    try {
      const result = await authService.logout();
      if (result.success) {
        await checkStatus();
        router.push(result.redirectUrl);
      }
    } catch (error) {
      console.error("Error signing out:", error);
    }
  };

  const renderNavItem = (item: ReturnType<typeof navigationService.getNavItems>[0]) => (
    <NavigationMenuItem key={item.key}>
      <Link href={item.href} legacyBehavior passHref>
        <Button
          variant="ghost"
          asChild
          className={`flex items-center gap-2 rounded-full hover:bg-primary/10
          ${
            pathname === item.href
              ? "bg-primary/10 text-primary"
              : "text-muted-foreground hover:text-primary"
          }`}
        >
          <NavigationMenuLink className="flex flex-row items-center gap-2">
            <div>{iconMap[item.key as keyof typeof iconMap]}</div>
            <div>{item.label}</div>
          </NavigationMenuLink>
        </Button>
      </Link>
    </NavigationMenuItem>
  );

  return (
    <div className="bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 border-b backdrop-blur-sm">
      <div className="flex h-20 items-center px-6 justify-between max-w-7xl mx-auto">
        <div className="flex items-center gap-8">
          <Link href="/" className="flex items-center gap-2 font-bold text-xl">
            <span className="bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
              EPSX
            </span>
          </Link>
          <NavigationMenu className="hidden md:block">
            <NavigationMenuList className="flex space-x-2">
              {navItems.map(renderNavItem)}
            </NavigationMenuList>
          </NavigationMenu>
        </div>

        <div className="flex items-center gap-4 md:gap-6">
          <ThemeToggle />

          {isLoggedIn && userEmail && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Link href="/settings" className="flex items-center gap-2">
                    <Avatar className="h-8 w-8">
                      <AvatarFallback>
                        <User className="h-4 w-4" />
                      </AvatarFallback>
                    </Avatar>
                    <span className="hidden md:inline text-muted-foreground hover:text-primary">
                      Settings
                    </span>
                  </Link>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{userEmail}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}

          {/* Mobile Menu */}
          <Sheet open={isOpen} onOpenChange={setIsOpen}>
            <SheetTrigger asChild className="md:hidden">
              <Button variant="ghost" size="icon">
                <Menu className="h-5 w-5" />
              </Button>
            </SheetTrigger>
            <SheetContent side="right">
              <SheetHeader>
                <SheetTitle className="text-left">Menu</SheetTitle>
              </SheetHeader>
              <div className="flex flex-col gap-4 mt-6">
                {navItems.map((item) => (
                  <Link
                    key={item.key}
                    href={item.href}
                    onClick={() => setIsOpen(false)}
                    className={`flex items-center gap-2 p-2 rounded-lg hover:bg-primary/10
                      ${pathname === item.href ? "bg-primary/10 text-primary" : "text-muted-foreground hover:text-primary"}`}
                  >
                    {iconMap[item.key as keyof typeof iconMap]}
                    {item.label}
                  </Link>
                ))}

                {isLoggedIn ? (
                  <Button
                    variant="ghost"
                    onClick={() => {
                      setIsOpen(false);
                      handleLogout();
                    }}
                    className="flex items-center gap-2 w-full justify-start p-2 rounded-lg hover:bg-primary/10 text-muted-foreground hover:text-primary mt-4"
                  >
                    <LogOut className="h-4 w-4" />
                    Logout
                  </Button>
                ) : (
                  <Link
                    href="/login"
                    onClick={() => setIsOpen(false)}
                    className="flex items-center gap-2 p-2 rounded-lg hover:bg-primary/10 text-muted-foreground hover:text-primary mt-4"
                  >
                    <LogIn className="h-4 w-4" />
                    Login
                  </Link>
                )}
              </div>
            </SheetContent>
          </Sheet>

          <div className="hidden md:block">
            {isLoggedIn ? (
              <Button
                variant="ghost"
                onClick={handleLogout}
                className="flex items-center gap-2 rounded-full hover:bg-primary/10"
              >
                <LogOut className="h-4 w-4" />
                Logout
              </Button>
            ) : (
              <Link href="/login">
                <Button
                  variant="ghost"
                  className="flex items-center gap-2 rounded-full hover:bg-primary/10"
                >
                  <LogIn className="h-4 w-4" />
                  Login
                </Button>
              </Link>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// Memoize the component to prevent unnecessary re-renders
export default memo(NavbarComponent);
