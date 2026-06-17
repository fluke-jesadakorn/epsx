import type { CountryOption } from './analytics';

export interface RichFilterOptions {
    countries: CountryOption[];
    sectors: string[];
    exchanges?: string[];
    stock_types?: string[];
}

export const DEFAULT_FILTER_OPTIONS: RichFilterOptions = {
    countries: [
        { value: 'america', label: 'United States', flag: '🇺🇸' },
        { value: 'korea', label: 'South Korea', flag: '🇰🇷' },
        { value: 'rsa', label: 'South Africa', flag: '🇿🇦' },
        { value: 'ksa', label: 'Saudi Arabia', flag: '🇸🇦' },
        { value: 'uae', label: 'United Arab Emirates', flag: '🇦🇪' },
        { value: 'newzealand', label: 'New Zealand', flag: '🇳🇿' },
        { value: 'hongkong', label: 'Hong Kong', flag: '🇭🇰' },
        { value: 'czech', label: 'Czech Republic', flag: '🇨🇿' },
        { value: 'srilanka', label: 'Sri Lanka', flag: '🇱🇰' },
        { value: 'uk', label: 'United Kingdom', flag: '🇬🇧' },
    ],
    sectors: [
        'Technology',
        'Financial Services',
        'Healthcare',
        'Consumer Cyclical',
        'Communication Services',
        'Industrials',
        'Consumer Defensive',
        'Energy',
        'Basic Materials',
        'Real Estate',
        'Utilities',
    ],
    exchanges: ['NYSE', 'NASDAQ', 'AMEX', 'OTC'],
    stock_types: ['Common Stock', 'ETF', 'ADR'],
};
