import type { Page } from '@playwright/test';

export interface Step {
  name: string;
  desc: string;
  action: (page: Page) => Promise<void>;
}

export interface Objective {
  id: string;
  name: string;
  route: string;
  auth: boolean;
  steps: Step[];
  mockOverrides?: Record<string, unknown>;
}

export interface MockHandler {
  pattern: string | RegExp;
  handler: (url: URL) => unknown;
}
