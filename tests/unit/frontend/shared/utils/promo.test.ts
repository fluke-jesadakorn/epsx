import { getTimeRemaining, resolvePromotionEndDate } from '@/shared/utils/promo';

describe('resolvePromotionEndDate', () => {
  it('falls back to one year from now when the promotion end date is blank', () => {
    const now = new Date('2026-04-20T03:25:45.234Z');

    expect(resolvePromotionEndDate('', now).toISOString()).toBe(
      '2027-04-20T03:25:45.234Z'
    );
  });

  it('keeps valid promotion end dates unchanged', () => {
    const now = new Date('2026-04-20T03:25:45.234Z');
    const expectedEndDate = '2026-12-31T00:00:00.000Z';

    expect(resolvePromotionEndDate(expectedEndDate, now).toISOString()).toBe(
      expectedEndDate
    );
  });
});

describe('getTimeRemaining', () => {
  beforeEach(() => {
    jest.useFakeTimers();
    jest.setSystemTime(new Date('2026-04-20T00:00:00.000Z'));
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('uses the one-year fallback window instead of returning NaN when the end date is blank', () => {
    expect(getTimeRemaining('')).toBe('365d left');
  });
});
