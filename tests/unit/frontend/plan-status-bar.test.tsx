import PlanStatusBar from '@/components/analytics/plan-status-bar';
import { useUpgradeOptions } from '@/hooks/use-upgrade-options';
import { render, screen } from '@testing-library/react';

// Mock hooks
jest.mock('@/hooks/use-upgrade-options');
jest.mock('next/link', () => {
    return ({ children, href }: { children: React.ReactNode; href: string }) => {
        return <a href={href}>{children}</a>;
    };
});

describe('plan-status-bar', () => {
    const mockUseUpgradeOptions = useUpgradeOptions as jest.Mock;

    beforeEach(() => {
        mockUseUpgradeOptions.mockReturnValue({
            nextPlan: null,
            recommendedPlan: null,
            loading: false,
            error: null
        });
    });

    it('renders null if planAccess is undefined', () => {
        const { container } = render(<PlanStatusBar />);
        expect(container.firstChild).toBeNull();
    });

    it('renders full access for ranking offset 1 (backend: offset=1 → skip=0 → rank 1 visible)', () => {
        const planAccess = {
            ranking_offset: 1,
            plan_name: 'Starter Plan',
            can_upgrade: false,
            tier_level: 1,
            wallet_address: '',
            plan_expires_at: null,
            days_remaining: 0,
            status: 'active' as const
        };

        render(<PlanStatusBar planAccess={planAccess} />);

        // offset=1 means full access (backend skips 0 results)
        expect(screen.getByText('All ranks (1+)')).toBeInTheDocument();
        expect(screen.getByText('Full Access')).toBeInTheDocument();
        expect(screen.queryByText(/locked/)).not.toBeInTheDocument();
        expect(screen.queryByText(/Unlock/)).not.toBeInTheDocument();
    });

    it('renders correct text for ranking offset > 1 (offset=5 → ranks 1-4 locked)', () => {
        const planAccess = {
            ranking_offset: 5,
            plan_name: 'Basic Plan',
            can_upgrade: true,
            tier_level: 1,
            wallet_address: '',
            plan_expires_at: null,
            days_remaining: 0,
            status: 'active' as const
        };

        render(<PlanStatusBar planAccess={planAccess} />);

        expect(screen.getByText('Ranks 1-4 locked')).toBeInTheDocument();
        expect(screen.getByText('Unlock ranks 1-4')).toBeInTheDocument();
    });

    it('renders correct text for ranking offset 2 (offset=2 → rank 1 locked)', () => {
        const planAccess = {
            ranking_offset: 2,
            plan_name: 'Basic Plan',
            can_upgrade: true,
            tier_level: 1,
            wallet_address: '',
            plan_expires_at: null,
            days_remaining: 0,
            status: 'active' as const
        };

        render(<PlanStatusBar planAccess={planAccess} />);

        expect(screen.getByText('Rank 1 locked')).toBeInTheDocument();
        expect(screen.getByText('Unlock Rank 1')).toBeInTheDocument();
    });

    it('displays next upgrade plan name in CTA for offset > 1', () => {
        const planAccess = {
            ranking_offset: 100,
            can_upgrade: true,
            plan_name: null,
            tier_level: 0,
            wallet_address: '',
            plan_expires_at: null,
            days_remaining: 0,
            status: 'active' as const
        };

        mockUseUpgradeOptions.mockReturnValue({
            nextPlan: { name: 'Pro Plan' },
            loading: false,
        });

        render(<PlanStatusBar planAccess={planAccess} />);

        expect(screen.getByText('Unlock ranks 1-99 with Pro Plan')).toBeInTheDocument();
    });

    it('handles full access (offset 0)', () => {
        const planAccess = {
            ranking_offset: 0,
            plan_name: 'Elite Plan',
            can_upgrade: false, // Enterprise/Max tier
            tier_level: 4,
            wallet_address: '',
            plan_expires_at: null,
            days_remaining: 0,
            status: 'active' as const
        };

        render(<PlanStatusBar planAccess={planAccess} />);

        expect(screen.getByText('All ranks (1+)')).toBeInTheDocument();
        expect(screen.getByText('Full Access')).toBeInTheDocument();
        expect(screen.queryByText(/locked/)).not.toBeInTheDocument();
    });
});
