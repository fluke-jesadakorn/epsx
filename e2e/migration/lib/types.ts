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

export type SessionAudience = 'epsx-frontend' | 'epsx-admin';

export interface ScenarioState {
  id: string;
  session: 'signed-out' | 'authenticated';
  audience?: SessionAudience;
  permissions?: string[];
  fixtureMode?: string;
  fixtureModeSide?: 'source' | 'target' | 'both';
  offline?: boolean;
}

export type ScenarioAction =
  | { type: 'click'; selector: string }
  | { type: 'fill'; selector: string; value: string }
  | { type: 'press'; selector: string; key: string }
  | { type: 'reload' }
  | { type: 'set-offline'; offline: boolean }
  | { type: 'wait-for'; selector: string };

export type ScenarioOutcome = (
  | { type: 'path'; value: string }
  | { type: 'query'; key: string; value: string }
  | { type: 'text'; value: string }
  | { type: 'text-absent'; value: string }
  | { type: 'selector'; value: string }
  | {
      type: 'attribute';
      selector: string;
      name: string;
      value: string;
    }
  | { type: 'focused'; selector: string }
  | { type: 'no-horizontal-overflow' }
  | { type: 'status'; value: number }
) & { side?: 'source' | 'target' | 'both' };

export interface Scenario {
  id: string;
  surface: Surface;
  path: string;
  title: string;
  state: ScenarioState;
  actions: ScenarioAction[];
  outcomes: ScenarioOutcome[];
  fixtureRequirements: string[];
  expectedSourcePath?: string;
  expectedTargetPath?: string;
}

export interface BackendContractSuite {
  id: string;
  title: string;
  executable: 'cargo';
  arguments: string[];
  claims: string[];
  sources: string[];
}

export interface ScenarioGroup {
  id: number;
  slug: string;
  title: string;
  matrix: string;
  repeat: number;
  comparisonGate: 'capture-only' | 'parity';
  surfaces: Surface[];
  states: string[];
  actions: string[];
  outcomes: string[];
  fixtureRequirements: string[];
  scenarios?: Scenario[];
  routes?: Partial<Record<Surface, string[]>>;
  includeAllContractRoutes?: boolean;
  browsers?: string[];
  requiredBypasses?: number;
  backendContracts?: BackendContractSuite[];
}

export interface ScenarioManifest {
  schemaVersion: number;
  baselineLock: string;
  routeContract: string;
  approvedDifferences: string;
  matrices: Record<string, ScenarioMatrix[]>;
  groups: ScenarioGroup[];
}

export type ApprovedDifferenceCategory =
  | 'backend-authority'
  | 'security'
  | 'wallet-siwe-legal-accuracy'
  | 'unsupported-feature-removal';

export interface ApprovedDifference {
  scenarioId: string;
  matrixIds: string[];
  category: ApprovedDifferenceCategory;
  reason: string;
  sourceEvidence: string;
  targetEvidence: string;
  maximumDifferencePercent: number;
}

export interface ApprovedDifferenceRegistry {
  schemaVersion: number;
  maximumUnapprovedDifferencePercent: number;
  allowedCategories: ApprovedDifferenceCategory[];
  items: ApprovedDifference[];
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
  outcomeChecks: Array<{
    outcome: ScenarioOutcome;
    passed: boolean;
    actual?: string | number | boolean;
  }>;
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
  approvalCategory?: ApprovedDifferenceCategory;
  maximumAllowedDifferencePercent: number;
}

export interface BackendContractRepeat {
  schemaVersion: number;
  groupId: number;
  suiteId: string;
  repeat: number;
  command: string[];
  startedAt: string;
  completedAt: string;
  durationMs: number;
  exitCode: number;
  passedTests: number;
  failedTests: number;
  ignoredTests: number;
  outputPath: string;
  outputSha256: string;
  preResetPath: string;
  postResetPath: string;
  passed: boolean;
}

export interface BackendContractReproducibility {
  schemaVersion: number;
  groupId: number;
  suiteId: string;
  title: string;
  repeats: number;
  claims: string[];
  sources: string[];
  results: BackendContractRepeat[];
  checks: {
    allRunsPassed: boolean;
    stablePassedTestCount: boolean;
    stableIgnoredTestCount: boolean;
    noIgnoredTests: boolean;
  };
  passed: boolean;
}
