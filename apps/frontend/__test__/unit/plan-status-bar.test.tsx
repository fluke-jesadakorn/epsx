import PlanStatusBar from '@/components/analytics/plan-status-bar';
import { usePlanAccess } from '@/hooks/use-plan-access';
import { useUpgradeOptions } from '@/hooks/use-upgrade-options';
import { render, screen } from '@testing-library/react';

// Mock hooks
jest.mock('@/hooks/use-plan-access');
jest.mock('@/hooks/use-upgrade-options');
jest.mock('next/link', () => {
    return ({ children, href }: { children: React.ReactNode; href: string }) => {
        return <a href={href}>{children}</a>;
    };
});

describe('plan-status-bar', () => {
    const mockUsePlanAccess = usePlanAccess as jest.Mock;
    const mockUseUpgradeOptions = useUpgradeOptions as jest.Mock;

    beforeEach(() => {
        mockUsePlanAccess.mockReturnValue({
            planAccess: null,
            loading: false,
        });
        mockUseUpgradeOptions.mockReturnValue({
            nextPlan: null,
            recommendedPlan: null,
            loading: false,
            error: null
        });
    });

    it('renders loading state', () => {
        mockUsePlanAccess.mockReturnValue({ loading: true });
        const { container } = render(<PlanStatusBar />);
        expect(container.firstChild).toHaveClass('animate-pulse');
    });

    it('renders correct text for ranking offset 1 (The "1-1" Fix)', () => {
        mockUsePlanAccess.mockReturnValue({
            planAccess: {
                ranking_offset: 1,
                plan_name: 'Starter Plan',
                can_upgrade: true,
                tier_level: 1
            },
            loading: false,
        });

        render(<PlanStatusBar />);

        // Check if "Rank 1 locked" text is present (instead of "Ranks 1-1 locked")
        expect(screen.getByText('Rank 1 locked')).toBeInTheDocument();

        // Check "Unlock Rank 1" in CTA
        expect(screen.getByText('Unlock Rank 1')).toBeInTheDocument();

        // Ensure "Unlock ranks 1-1" is NOT present
        expect(screen.queryByText('Unlock ranks 1-1')).not.toBeInTheDocument();
    });

    it('renders correct text for ranking offset > 1', () => {
        mockUsePlanAccess.mockReturnValue({
            planAccess: {
                ranking_offset: 5,
                plan_name: 'Basic Plan',
                can_upgrade: true,
                tier_level: 1
            },
            loading: false,
        });

        render(<PlanStatusBar />);

        expect(screen.getByText('Ranks 1-5 locked')).toBeInTheDocument();
        expect(screen.getByText('Unlock ranks 1-5')).toBeInTheDocument();
    });

    it('displays next upgrade plan name in CTA', () => {
        mockUsePlanAccess.mockReturnValue({
            planAccess: {
                ranking_offset: 1,
                can_upgrade: true,
            },
            loading: false,
        });

        mockUseUpgradeOptions.mockReturnValue({
            nextPlan: { name: 'Pro Plan' },
            loading: false,
        });

        render(<PlanStatusBar />);

        expect(screen.getByText('Unlock Rank 1 with Pro Plan')).toBeInTheDocument();
    });

    it('handles full access (offset 0)', () => {
        mockUsePlanAccess.mockReturnValue({
            planAccess: {
                ranking_offset: 0,
                plan_name: 'Elite Plan',
                can_upgrade: false, // Enterprise/Max tier
            },
            loading: false,
        });

        render(<PlanStatusBar />);

        expect(screen.getByText('All ranks (1+)')).toBeInTheDocument();
        expect(screen.getByText('Full Access')).toBeInTheDocument();
        expect(screen.queryByText(/locked/)).not.toBeInTheDocument();
    });
});
