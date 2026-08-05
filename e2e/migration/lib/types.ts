export type Surface = 'frontend' | 'admin';
export type ColorScheme = 'light' | 'dark';

export interface Viewport {
  width: number;
  height: number;
}

export interface ScenarioMatrix {
  id: string;
  viewport: Viewport;
  colorScheme: ColorScheme;
}

export interface Scenario {
  id: string;
  surface: Surface;
  path: string;
  title: string;
  expectedSourcePath?: string;
  expectedTargetPath?: string;
}

export interface ScenarioGroup {
  id: number;
  slug: string;
  title: string;
  matrix?: string;
  repeat?: number;
  comparisonGate?: 'capture-only' | 'parity';
  scenarios?: Scenario[];
  routes?: Partial<Record<Surface, string[]>>;
  includeAllContractRoutes?: boolean;
  browsers?: string[];
  requiredBypasses?: number;
}

export interface ScenarioManifest {
  schemaVersion: number;
  baselineLock: string;
  routeContract: string;
  matrices: Record<string, ScenarioMatrix[]>;
  groups: ScenarioGroup[];
}

export interface BaselineLock {
  schemaVersion: number;
  repository: string;
  ref: string;
  commit: string;
  application: string;
  packageManager: string;
  dependencyLock: string;
  dependencyLockSha256: string;
  immutable: boolean;
}

export interface RuntimeConfig {
  repoRoot: string;
  sourceRoot: string;
  runRoot: string;
  artifactRoot: string;
  evidenceRoot: string;
  groupId: number;
  sourceCommit: string;
  targetCommit: string;
  sourceFrontendUrl: string;
  sourceAdminUrl: string;
  targetFrontendUrl: string;
  targetAdminUrl: string;
  fixtureUrl: string;
  fixtureToken: string;
  postgresAdminUrl: string;
  postgresTemplateDatabase: string;
  postgresRuntimeDatabase: string;
  redisUrl: string;
  redisPrefix: string;
  anvilUrl: string;
  allowRuntimeMutation: boolean;
}

export interface StateDigest {
  capturedAt: string;
  postgres: {
    database: string;
    tables: Array<{ table: string; rows: number; checksum: string }>;
    checksum: string;
    transientTablesEmpty: boolean;
  };
  redis: {
    prefix: string;
    keys: Array<{ key: string; valueHash: string }>;
    checksum: string;
  };
  anvil: {
    chainId: string;
    blockNumber: string;
    accounts: Array<{
      address: string;
      balance: string;
      nonce: string;
      codeHash: string;
    }>;
    checksum: string;
  };
  fixture: {
    requestCount: number;
    requests: unknown[];
    mutations: unknown[];
    checksum: string;
  };
}

export interface ResetProof {
  schemaVersion: number;
  scenarioId: string;
  phase: 'pre' | 'post';
  startedAt: string;
  completedAt: string;
  beforeReset: StateDigest;
  afterReset: StateDigest;
  baseline: StateDigest;
  checks: {
    postgresMatchesBaseline: boolean;
    transientTablesEmpty: boolean;
    redisMatchesBaseline: boolean;
    anvilMatchesBaseline: boolean;
    fixtureMatchesBaseline: boolean;
  };
  passed: boolean;
}

export interface BrowserLogEntry {
  type: string;
  text: string;
  location?: string;
}

export interface NetworkEntry {
  kind: 'request' | 'response' | 'failed';
  method?: string;
  status?: number;
  resourceType?: string;
  url: string;
  failure?: string;
}

export interface CaptureResult {
  side: 'source' | 'target';
  scenarioId: string;
  matrixId: string;
  repeat: number;
  requestedUrl: string;
  finalUrl: string;
  status: number | null;
  title: string;
  bodyTextLength: number;
  consoleErrors: BrowserLogEntry[];
  pageErrors: string[];
  failedRequests: NetworkEntry[];
  artifactDirectory: string;
  screenshotPath: string;
  domPath: string;
  normalizedDomPath: string;
  accessibilityPath: string;
  networkPath: string;
  browserLogPath: string;
  redirectsPath: string;
  tracePath: string;
  videoPath?: string;
  harPath: string;
  browserResetPath: string;
  screenshotSha256: string;
  domSha256: string;
  accessibilitySha256: string;
}

export interface ComparisonResult {
  schemaVersion: number;
  scenarioId: string;
  matrixId: string;
  repeat: number;
  sourceScreenshot: string;
  targetScreenshot: string;
  diffScreenshot: string;
  contactSheet: string;
  differingPixels: number;
  totalPixels: number;
  differencePercent: number;
  approvedDifference: boolean;
  approvalReason: string;
}
