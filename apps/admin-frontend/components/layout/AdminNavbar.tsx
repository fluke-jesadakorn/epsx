'use client';

/**
 * ADMIN NAVBAR
 * Horizontal navigation matching frontend design
 * Replaces the old Header + Sidebar layout
 */

import { ConnectButton } from '@rainbow-me/rainbowkit';
import {
    BarChart3,
    ChevronDown,
    CreditCard,
    FileText,
    Key,
    LayoutDashboard,
    Link as LinkIcon,
    LogOut,
    Menu,
    Settings,
    Shield,
    Users,
    Wallet,
    X
} from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useAccount, useChainId, useDisconnect, useSwitchChain } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { isProduction } from '@/shared/utils';

// Navigation items for admin
const adminNavItems = [
    {
        key: 'dashboard',
        label: 'Dashboard',
        href: '/',
        icon: LayoutDashboard,
    },
    {
        key: 'management',
        label: 'Management',
        icon: Users,
        hasDropdown: true,
        children: [
            { key: 'users', label: 'Users', href: '/wallet-management', icon: Wallet },
            { key: 'groups', label: 'Groups', href: '/groups', icon: Users },
            { key: 'permissions', label: 'Permissions', href: '/permissions', icon: Shield },
        ],
    },
    {
        key: 'subscriptions',
        label: 'Subscriptions',
        icon: CreditCard,
        hasDropdown: true,
        children: [
            { key: 'plans', label: 'Plans', href: '/plans', icon: FileText },
            { key: 'subs', label: 'All Subscriptions', href: '/subscriptions', icon: CreditCard },
            { key: 'payments', label: 'Payments', href: '/payments', icon: Wallet },
        ],
    },
    {
        key: 'developer',
        label: 'Developer',
        icon: Key,
        hasDropdown: true,
        children: [
            { key: 'api-keys', label: 'API Keys', href: '/developer-portal/api-keys', icon: Key },
            { key: 'analytics', label: 'Analytics', href: '/analytics', icon: BarChart3 },
        ],
    },
];

export function AdminNavbar() {
    const pathname = usePathname();
    const [isOpen, setIsOpen] = useState(false);
    const [mounted, setMounted] = useState(false);
    const { logout } = useSharedAuth();

    const chainId = useChainId();
    const { switchChain, isPending: isSwitching } = useSwitchChain();
    const { isConnected, address } = useAccount();
    const { disconnect } = useDisconnect();

    useEffect(() => {
        setMounted(true);
    }, []);

    const getCurrentChainName = () => {
        if (!mounted) return 'Chain';
        if (chainId === bsc.id) return 'BSC Mainnet';
        if (chainId === bscTestnet.id) return 'BSC Testnet';
        if (chainId === 31337) return 'Hardhat Local';
        return 'Unknown Chain';
    };

    const handleChainSwitch = async (targetChainId: number) => {
        if (!isConnected || isSwitching || targetChainId === chainId) return;
        try {
            await switchChain({ chainId: targetChainId });
        } catch (error) {
            console.error('Failed to switch chain:', error);
        }
    };

    const handleDisconnect = async () => {
        try {
            await logout();
            disconnect();
        } catch (error) {
            console.error('Disconnect error:', error);
        }
    };

    const formatAddress = (addr: string) => `${addr.slice(0, 6)}...${addr.slice(-4)}`;

    // Skeleton during SSR
    if (!mounted) {
        return (
            <header className="sticky top-0 z-50 border-b border-slate-700/50 bg-slate-900/95 backdrop-blur-xl">
                <div className="mx-auto flex h-16 items-center px-6">
                    <Link href="/" className="flex items-center mr-8">
                        <span className="text-xl font-bold bg-gradient-to-r from-orange-400 to-yellow-500 bg-clip-text text-transparent">EPSX</span>
                        <span className="ml-2 text-xs text-slate-400 font-medium">ADMIN</span>
                    </Link>
                    <div className="flex-1" />
                    <div className="h-8 w-24 bg-slate-700 rounded-lg animate-pulse" />
                </div>
            </header>
        );
    }

    return (
        <header className="sticky top-0 z-50 border-b border-slate-700/50 bg-slate-900/95 backdrop-blur-xl">
            <div className="mx-auto flex h-16 items-center px-6">
                {/* Logo */}
                <Link href="/" className="flex items-center mr-8 hover:opacity-80 transition-opacity">
                    <span className="text-xl font-bold bg-gradient-to-r from-orange-400 to-yellow-500 bg-clip-text text-transparent">EPSX</span>
                    <span className="ml-2 text-xs text-slate-400 font-medium uppercase tracking-wider">Admin</span>
                </Link>

                {/* Main Navigation - Desktop */}
                <nav className="hidden items-center gap-1 lg:flex">
                    {adminNavItems.map(item => {
                        const IconComponent = item.icon;
                        const isActive = pathname === item.href || item.children?.some(child => pathname === child.href);

                        if (item.hasDropdown && item.children) {
                            return (
                                <DropdownMenu key={item.key}>
                                    <DropdownMenuTrigger asChild>
                                        <button
                                            className={`flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${isActive
                                                    ? 'bg-orange-500/20 text-orange-400'
                                                    : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                                                }`}
                                        >
                                            <IconComponent className="h-4 w-4 text-orange-500" />
                                            {item.label}
                                            <ChevronDown className="h-3 w-3 text-slate-400" />
                                        </button>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent
                                        align="start"
                                        className="w-48 bg-slate-900 border-slate-700 p-1"
                                        style={{ zIndex: 99999 }}
                                    >
                                        {item.children.map(child => {
                                            const ChildIcon = child.icon;
                                            return (
                                                <DropdownMenuItem key={child.key} asChild>
                                                    <Link
                                                        href={child.href}
                                                        className={`flex items-center gap-2 px-3 py-2 rounded-md text-sm cursor-pointer ${pathname === child.href
                                                                ? 'bg-orange-500/20 text-orange-400'
                                                                : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                                                            }`}
                                                    >
                                                        <ChildIcon className="h-4 w-4 text-orange-500" />
                                                        {child.label}
                                                    </Link>
                                                </DropdownMenuItem>
                                            );
                                        })}
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            );
                        }

                        return (
                            <Link
                                key={item.key}
                                href={item.href!}
                                className={`flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${pathname === item.href
                                        ? 'bg-orange-500/20 text-orange-400'
                                        : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                                    }`}
                            >
                                <IconComponent className="h-4 w-4 text-orange-500" />
                                {item.label}
                            </Link>
                        );
                    })}
                </nav>

                {/* Spacer */}
                <div className="flex-1" />

                {/* Right Actions */}
                <div className="hidden items-center gap-2 lg:flex">
                    {/* Settings */}
                    <Link
                        href="/settings"
                        className={`flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${pathname === '/settings'
                                ? 'bg-orange-500/20 text-orange-400'
                                : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                            }`}
                    >
                        <Settings className="h-4 w-4 text-orange-500" />
                        <span>Settings</span>
                    </Link>

                    {/* Chain Selector - Only in dev */}
                    {!isProduction && (
                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <button
                                    className="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
                                    disabled={isSwitching || !isConnected}
                                >
                                    <LinkIcon className="h-4 w-4 text-orange-500" />
                                    <span>{getCurrentChainName()}</span>
                                    <ChevronDown className="h-3 w-3 text-slate-400" />
                                </button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent
                                align="end"
                                className="w-48 bg-slate-900 border-slate-700 p-1"
                                style={{ zIndex: 99999 }}
                            >
                                <DropdownMenuItem
                                    onClick={() => handleChainSwitch(bsc.id)}
                                    disabled={chainId === bsc.id}
                                    className={`flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer ${chainId === bsc.id ? 'bg-orange-500/20 text-orange-400' : 'text-slate-300 hover:bg-slate-800'
                                        }`}
                                >
                                    <div className="w-5 h-5 rounded-full bg-yellow-500 flex items-center justify-center text-[10px] font-bold text-white">56</div>
                                    BSC Mainnet
                                </DropdownMenuItem>
                                <DropdownMenuItem
                                    onClick={() => handleChainSwitch(bscTestnet.id)}
                                    disabled={chainId === bscTestnet.id}
                                    className={`flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer ${chainId === bscTestnet.id ? 'bg-orange-500/20 text-orange-400' : 'text-slate-300 hover:bg-slate-800'
                                        }`}
                                >
                                    <div className="w-5 h-5 rounded-full bg-purple-500 flex items-center justify-center text-[10px] font-bold text-white">97</div>
                                    BSC Testnet
                                </DropdownMenuItem>
                            </DropdownMenuContent>
                        </DropdownMenu>
                    )}

                    {/* Wallet Connect */}
                    {isConnected ? (
                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <button className="flex items-center gap-2 rounded-lg bg-gradient-to-r from-orange-500 to-yellow-500 px-4 py-2 text-sm font-medium text-white hover:opacity-90 transition-opacity">
                                    <Wallet className="h-4 w-4" />
                                    <span>{formatAddress(address!)}</span>
                                    <ChevronDown className="h-3 w-3" />
                                </button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent
                                align="end"
                                className="w-48 bg-slate-900 border-slate-700 p-1"
                                style={{ zIndex: 99999 }}
                            >
                                <div className="px-3 py-2 text-xs text-slate-400 border-b border-slate-700 mb-1">
                                    {address}
                                </div>
                                <DropdownMenuItem
                                    onClick={handleDisconnect}
                                    className="flex items-center gap-2 px-3 py-2 rounded-md text-red-400 hover:bg-red-500/20 cursor-pointer"
                                >
                                    <LogOut className="h-4 w-4" />
                                    Disconnect
                                </DropdownMenuItem>
                            </DropdownMenuContent>
                        </DropdownMenu>
                    ) : (
                        <ConnectButton.Custom>
                            {({ openConnectModal }) => (
                                <button
                                    onClick={openConnectModal}
                                    className="flex items-center gap-2 rounded-lg bg-gradient-to-r from-orange-500 to-yellow-500 px-4 py-2 text-sm font-medium text-white hover:opacity-90 transition-opacity"
                                >
                                    <Wallet className="h-4 w-4" />
                                    <span>Connect Wallet</span>
                                </button>
                            )}
                        </ConnectButton.Custom>
                    )}
                </div>

                {/* Mobile Menu */}
                <Sheet open={isOpen} onOpenChange={setIsOpen}>
                    <SheetTrigger asChild className="lg:hidden">
                        <button className="rounded-lg bg-slate-800 p-2 text-slate-300 hover:bg-slate-700">
                            <Menu className="h-5 w-5" />
                        </button>
                    </SheetTrigger>
                    <SheetContent side="right" className="w-80 bg-slate-900 border-slate-700 p-0">
                        <SheetHeader className="p-4 border-b border-slate-700">
                            <div className="flex items-center justify-between">
                                <div className="flex items-center">
                                    <span className="text-xl font-bold bg-gradient-to-r from-orange-400 to-yellow-500 bg-clip-text text-transparent">EPSX</span>
                                    <span className="ml-2 text-xs text-slate-400 font-medium">ADMIN</span>
                                </div>
                                <button onClick={() => setIsOpen(false)} className="text-slate-400 hover:text-white">
                                    <X className="h-5 w-5" />
                                </button>
                            </div>
                            <SheetTitle className="sr-only">Navigation Menu</SheetTitle>
                        </SheetHeader>

                        <div className="p-4 space-y-2">
                            {adminNavItems.map(item => {
                                const IconComponent = item.icon;
                                const isActive = pathname === item.href || item.children?.some(child => pathname === child.href);

                                return (
                                    <div key={item.key}>
                                        {item.hasDropdown ? (
                                            <>
                                                <div className={`flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium ${isActive ? 'text-orange-400' : 'text-slate-300'
                                                    }`}>
                                                    <IconComponent className="h-4 w-4 text-orange-500" />
                                                    {item.label}
                                                </div>
                                                <div className="ml-7 space-y-1">
                                                    {item.children?.map(child => {
                                                        const ChildIcon = child.icon;
                                                        return (
                                                            <Link
                                                                key={child.key}
                                                                href={child.href}
                                                                onClick={() => setIsOpen(false)}
                                                                className={`flex items-center gap-2 rounded-lg px-3 py-2 text-sm ${pathname === child.href
                                                                        ? 'bg-orange-500/20 text-orange-400'
                                                                        : 'text-slate-400 hover:bg-slate-800 hover:text-white'
                                                                    }`}
                                                            >
                                                                <ChildIcon className="h-4 w-4" />
                                                                {child.label}
                                                            </Link>
                                                        );
                                                    })}
                                                </div>
                                            </>
                                        ) : (
                                            <Link
                                                href={item.href!}
                                                onClick={() => setIsOpen(false)}
                                                className={`flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium ${pathname === item.href
                                                        ? 'bg-orange-500/20 text-orange-400'
                                                        : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                                                    }`}
                                            >
                                                <IconComponent className="h-4 w-4 text-orange-500" />
                                                {item.label}
                                            </Link>
                                        )}
                                    </div>
                                );
                            })}

                            {/* Mobile Wallet Section */}
                            <div className="pt-4 border-t border-slate-700 mt-4">
                                {isConnected ? (
                                    <button
                                        onClick={handleDisconnect}
                                        className="w-full flex items-center justify-center gap-2 rounded-lg bg-red-500/20 text-red-400 px-4 py-3 font-medium"
                                    >
                                        <LogOut className="h-4 w-4" />
                                        Disconnect ({formatAddress(address!)})
                                    </button>
                                ) : (
                                    <ConnectButton.Custom>
                                        {({ openConnectModal }) => (
                                            <button
                                                onClick={openConnectModal}
                                                className="w-full flex items-center justify-center gap-2 rounded-lg bg-gradient-to-r from-orange-500 to-yellow-500 px-4 py-3 font-medium text-white"
                                            >
                                                <Wallet className="h-4 w-4" />
                                                Connect Wallet
                                            </button>
                                        )}
                                    </ConnectButton.Custom>
                                )}
                            </div>
                        </div>
                    </SheetContent>
                </Sheet>
            </div>
        </header>
    );
}

export default AdminNavbar;
