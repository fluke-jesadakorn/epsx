import {
  ActivityIcon,
  AlertTriangleIcon,
  BarChart3Icon,
  CheckCircleIcon,
  ClockIcon,
  DownloadIcon,
  FilterIcon,
  PauseIcon,
  PlayIcon,
  RefreshCwIcon,
  ShieldIcon,
  TrendingDownIcon,
  TrendingUpIcon,
  UserIcon,
  ZapIcon,
} from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import type { PolicyEvaluation, PolicyStats } from '@/hooks/use-policy-monitor';

const DECISION_COLORS = {
  allow: 'green',
  deny: 'red',
  require_mfa: 'yellow',
  require_approval: 'orange',
  restricted_access: 'purple',
};

const DECISION_ICONS = {
  allow: CheckCircleIcon,
  deny: AlertTriangleIcon,
  require_mfa: ShieldIcon,
  require_approval: ClockIcon,
  restricted_access: FilterIcon,
};

interface HeaderSectionProps {
  isLiveMode: boolean;
  lastUpdate: Date;
  onToggleLiveMode: () => void;
  onRefresh: () => void;
}

export function HeaderSection({ isLiveMode, lastUpdate, onToggleLiveMode, onRefresh }: HeaderSectionProps) {
  return (
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
      <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-4 p-5">
        <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-[14px] text-[#1fc7d4] border border-[#1fc7d4]/20">
              <ActivityIcon className="w-4 h-4" />
            </div>
            <h2 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent">Policy Monitoring Dashboard</h2>
          </div>
          <Badge variant={isLiveMode ? "default" : "outline"} className="flex items-center gap-1 w-fit">
            {isLiveMode ? <span className="w-2 h-2 bg-[#31d0aa] rounded-full" /> : null}
            {isLiveMode ? 'LIVE' : 'PAUSED'}
          </Badge>
        </div>

        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 w-full lg:w-auto">
          <span className="text-sm text-muted-foreground text-center sm:text-left">
            Last update: {lastUpdate.toLocaleTimeString()}
          </span>

          <div className="flex flex-col sm:flex-row gap-2 sm:gap-3">
            <Button
              variant="outline"
              size="sm"
              onClick={onToggleLiveMode}
              className="min-h-[44px] justify-center rounded-xl border-border/40"
            >
              {isLiveMode ? (
                <>
                  <PauseIcon className="h-4 w-4 mr-2" />
                  Pause
                </>
              ) : (
                <>
                  <PlayIcon className="h-4 w-4 mr-2" />
                  Resume
                </>
              )}
            </Button>

            <Button
              variant="outline"
              size="sm"
              onClick={onRefresh}
              className="min-h-[44px] justify-center rounded-xl border-border/40"
            >
              <RefreshCwIcon className="h-4 w-4 mr-2" />
              Refresh
            </Button>

            <Button
              size="sm"
              disabled
              className="min-h-[44px] justify-center rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white border-0"
            >
              <DownloadIcon className="h-4 w-4 mr-2" />
              Export Log
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

interface StatsOverviewProps {
  stats: PolicyStats;
}

function calculatePercentage(value: number, total: number) {
  return total > 0 ? Math.round((value / total) * 100) : 0;
}

export function StatsOverview({ stats }: StatsOverviewProps) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-3 sm:gap-4">
      <div className="rounded-2xl bg-card border border-border/20 p-3 sm:p-4 shadow-xl">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-xl text-[#1fc7d4] border border-[#1fc7d4]/20 flex-shrink-0">
            <ShieldIcon className="h-3.5 w-3.5" />
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Active Policies</p>
            <p className="text-lg sm:text-xl font-semibold text-foreground">{stats.active_policies}</p>
          </div>
        </div>
      </div>

      <div className="rounded-2xl bg-card border border-border/20 p-3 sm:p-4 shadow-xl">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-gradient-to-br from-[#31d0aa]/10 to-[#1fc7d4]/10 rounded-xl text-[#31d0aa] border border-[#31d0aa]/20 flex-shrink-0">
            <BarChart3Icon className="h-3.5 w-3.5" />
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Evaluations (24h)</p>
            <p className="text-lg sm:text-xl font-semibold text-foreground">{stats.evaluations_24h}</p>
          </div>
        </div>
      </div>

      <div className="rounded-2xl bg-card border border-border/20 p-3 sm:p-4 shadow-xl">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-xl text-[#ffb237] border border-[#ffb237]/20 flex-shrink-0">
            <ZapIcon className="h-3.5 w-3.5" />
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Avg Response</p>
            <p className="text-lg sm:text-xl font-semibold text-foreground">{Math.round(stats.avg_evaluation_time_ms)}ms</p>
          </div>
        </div>
      </div>

      <div className="rounded-2xl bg-card border border-border/20 p-3 sm:p-4 shadow-xl">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-gradient-to-br from-[#31d0aa]/10 to-[#1fc7d4]/10 rounded-xl text-[#31d0aa] border border-[#31d0aa]/20 flex-shrink-0">
            <CheckCircleIcon className="h-3.5 w-3.5" />
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Allowed</p>
            <p className="text-lg sm:text-xl font-semibold text-foreground">
              {calculatePercentage(stats.decision_breakdown.allow ?? 0, stats.evaluations_24h)}%
            </p>
          </div>
        </div>
      </div>

      <div className="rounded-2xl bg-card border border-border/20 p-3 sm:p-4 shadow-xl">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-gradient-to-br from-[#ed4b9e]/10 to-[#7645d9]/10 rounded-xl text-[#ed4b9e] border border-[#ed4b9e]/20 flex-shrink-0">
            <AlertTriangleIcon className="h-3.5 w-3.5" />
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Blocked</p>
            <p className="text-lg sm:text-xl font-semibold text-foreground">
              {calculatePercentage(stats.decision_breakdown.deny ?? 0, stats.evaluations_24h)}%
            </p>
          </div>
        </div>
      </div>

      <div className="rounded-2xl bg-card border border-border/20 p-3 sm:p-4 shadow-xl">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-xl text-[#ffb237] border border-[#ffb237]/20 flex-shrink-0">
            <ClockIcon className="h-3.5 w-3.5" />
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Pending</p>
            <p className="text-lg sm:text-xl font-semibold text-foreground">
              {calculatePercentage(
                (stats.decision_breakdown.require_approval ?? 0) + (stats.decision_breakdown.require_mfa ?? 0),
                stats.evaluations_24h
              )}%
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function getDecisionBadge(decision: string) {
  const _color = DECISION_COLORS[decision as keyof typeof DECISION_COLORS] ?? 'gray';
  const Icon = DECISION_ICONS[decision as keyof typeof DECISION_ICONS] ?? ActivityIcon;

  let variant: "default" | "destructive" | "outline" | "secondary" = "outline";
  if (decision === 'allow') { variant = 'default'; }
  if (decision === 'deny') { variant = 'destructive'; }
  if (decision === 'require_mfa') { variant = 'secondary'; }

  return (
    <Badge variant={variant} className="flex items-center gap-1 text-xs">
      <Icon className="h-3 w-3" />
      {decision.replace('_', ' ').toUpperCase()}
    </Badge>
  );
}

function formatTime(dateString: string) {
  const date = new Date(dateString);
  return date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
}

interface LiveEvaluationsProps {
  evaluations: PolicyEvaluation[];
  isLiveMode: boolean;
}

export function LiveEvaluations({ evaluations, isLiveMode }: LiveEvaluationsProps) {
  return (
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between p-5 border-b border-border/20 gap-3">
        <div className="flex flex-col sm:flex-row sm:items-center gap-2">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-[14px] text-[#1fc7d4] border border-[#1fc7d4]/20">
              <ActivityIcon className="w-4 h-4" />
            </div>
            <h3 className="text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]">Live Policy Evaluations</h3>
          </div>
          {isLiveMode && (
            <span className="text-sm text-muted-foreground">(refreshing every 2s)</span>
          )}
        </div>

        <div className="flex items-center gap-3 w-full sm:w-auto">
          <Button variant="outline" size="sm" disabled className="w-full sm:w-auto min-h-[44px] rounded-xl border-border/40">
            <FilterIcon className="h-4 w-4 mr-2" />
            Filter
          </Button>
        </div>
      </div>

      <div className="p-5">
        <div className="space-y-2 max-h-96 overflow-y-auto">
          <div className="hidden lg:grid grid-cols-12 gap-2 px-3 py-2 text-xs font-medium text-muted-foreground border-b border-border/20">
            <div className="col-span-1">Time</div>
            <div className="col-span-2">User</div>
            <div className="col-span-3">Action</div>
            <div className="col-span-2">Policy</div>
            <div className="col-span-2">Decision</div>
            <div className="col-span-2">Details</div>
          </div>

          {evaluations.map((evaluation, index) => (
            <div key={evaluation.id}>
              <div
                className={`hidden lg:grid grid-cols-12 gap-2 px-3 py-2 text-sm hover:bg-muted/30 rounded-xl ${index === 0 && isLiveMode ? 'bg-[#1fc7d4]/5 border-l-4 border-[#1fc7d4]' : ''
                  }`}
              >
                <div className="col-span-1 font-mono text-xs text-muted-foreground">
                  {formatTime(evaluation.evaluated_at)}
                </div>

                <div className="col-span-2">
                  <div className="flex items-center gap-1">
                    <UserIcon className="h-3 w-3 text-muted-foreground" />
                    <span className="truncate text-foreground">{evaluation.user_email}</span>
                  </div>
                </div>

                <div className="col-span-3">
                  <span className="font-mono text-xs bg-muted/30 border border-border/40 px-2 py-1 rounded-lg text-foreground">
                    {evaluation.action_attempted}
                  </span>
                </div>

                <div className="col-span-2">
                  <span className="text-xs text-muted-foreground">
                    {evaluation.policy_name ?? 'Unknown'}
                  </span>
                </div>

                <div className="col-span-2">
                  {getDecisionBadge(evaluation.decision)}
                </div>

                <div className="col-span-2">
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <ClockIcon className="h-3 w-3" />
                    <span>{evaluation.evaluation_time_ms}ms</span>
                  </div>
                </div>
              </div>

              <div className={`lg:hidden p-3 rounded-xl border bg-card hover:bg-muted/30 transition-colors ${index === 0 && isLiveMode ? 'bg-[#1fc7d4]/5 border-[#1fc7d4]/30' : 'border-border/40'
                }`}>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <UserIcon className="h-4 w-4 text-muted-foreground" />
                    <span className="font-medium text-sm text-foreground truncate">{evaluation.user_email}</span>
                  </div>
                  <span className="font-mono text-xs text-muted-foreground">
                    {formatTime(evaluation.evaluated_at)}
                  </span>
                </div>

                <div className="mb-2">
                  <span className="font-mono text-xs bg-muted/30 border border-border/40 px-2 py-1 rounded-lg text-foreground break-all">
                    {evaluation.action_attempted}
                  </span>
                </div>

                <div className="flex items-center justify-between">
                  <div className="flex flex-col gap-1">
                    <span className="text-xs text-muted-foreground">
                      {evaluation.policy_name ?? 'Unknown'}
                    </span>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <ClockIcon className="h-3 w-3" />
                      <span>{evaluation.evaluation_time_ms}ms</span>
                    </div>
                  </div>
                  <div>
                    {getDecisionBadge(evaluation.decision)}
                  </div>
                </div>
              </div>
            </div>
          ))}

          {evaluations.length === 0 && (
            <div className="text-center py-8 sm:py-12 text-muted-foreground">
              <div className="h-16 w-16 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 border border-[#1fc7d4]/20 rounded-2xl flex items-center justify-center mx-auto mb-4">
                <ActivityIcon className="h-8 w-8 text-[#1fc7d4]" />
              </div>
              <p className="text-base sm:text-lg font-medium">No recent evaluations</p>
              <p className="text-sm">Policy evaluations will appear here in real-time</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

interface ChartsGridProps {
  stats: PolicyStats;
}

export function ChartsGrid({ stats }: ChartsGridProps) {
  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 sm:gap-6">
      <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
        <div className="h-[3px] bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]" />
        <div className="flex items-center gap-3 p-5 border-b border-border/20">
          <div className="p-2 bg-gradient-to-br from-[#7645d9]/10 to-[#1fc7d4]/10 rounded-[14px] text-[#7645d9] border border-[#7645d9]/20">
            <BarChart3Icon className="w-4 h-4" />
          </div>
          <h3 className="text-xs font-bold text-[#7645d9] uppercase tracking-[0.2em]">Decision Breakdown (24h)</h3>
        </div>

        <div className="p-5 space-y-3">
          {Object.entries(stats.decision_breakdown).map(([decision, count]) => {
            const percentage = calculatePercentage(count, stats.evaluations_24h);
            const Icon = DECISION_ICONS[decision as keyof typeof DECISION_ICONS] ?? ActivityIcon;

            return (
              <div key={decision} className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 sm:gap-0">
                <div className="flex items-center gap-2">
                  <Icon className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm text-foreground capitalize">{decision.replace('_', ' ')}</span>
                </div>

                <div className="flex items-center gap-2">
                  <div className="w-20 sm:w-24 bg-muted/50 rounded-full h-2">
                    <div
                      className="bg-gradient-to-r from-[#7645d9] to-[#1fc7d4] h-2 rounded-full"
                      style={{ width: `${percentage}%` }}
                    />
                  </div>
                  <span className="text-sm font-medium text-foreground w-16 sm:w-12 text-right">
                    {count} ({percentage}%)
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
        <div className="h-[3px] bg-gradient-to-r from-[#31d0aa] to-[#1fc7d4]" />
        <div className="flex items-center gap-3 p-5 border-b border-border/20">
          <div className="p-2 bg-gradient-to-br from-[#31d0aa]/10 to-[#1fc7d4]/10 rounded-[14px] text-[#31d0aa] border border-[#31d0aa]/20">
            <TrendingUpIcon className="w-4 h-4" />
          </div>
          <h3 className="text-xs font-bold text-[#31d0aa] uppercase tracking-[0.2em]">Performance Trends</h3>
        </div>

        <div className="p-5 space-y-3 sm:space-y-4">
          <div className="flex items-center justify-between p-3 bg-[#31d0aa]/5 border border-[#31d0aa]/20 rounded-xl">
            <div className="flex items-center gap-2">
              <TrendingUpIcon className="h-4 w-4 text-[#31d0aa]" />
              <span className="text-sm font-medium text-foreground">Policy Efficiency</span>
            </div>
            <div className="text-right">
              <div className="text-base sm:text-lg font-semibold text-[#31d0aa]">+12%</div>
              <div className="text-xs text-muted-foreground">vs last week</div>
            </div>
          </div>

          <div className="flex items-center justify-between p-3 bg-[#1fc7d4]/5 border border-[#1fc7d4]/20 rounded-xl">
            <div className="flex items-center gap-2">
              <ZapIcon className="h-4 w-4 text-[#1fc7d4]" />
              <span className="text-sm font-medium text-foreground">Response Time</span>
            </div>
            <div className="text-right">
              <div className="text-base sm:text-lg font-semibold text-[#1fc7d4]">{Math.round(stats.avg_evaluation_time_ms)}ms</div>
              <div className="text-xs text-muted-foreground">avg evaluation</div>
            </div>
          </div>

          <div className="flex items-center justify-between p-3 bg-[#ffb237]/5 border border-[#ffb237]/20 rounded-xl">
            <div className="flex items-center gap-2">
              <TrendingDownIcon className="h-4 w-4 text-[#ffb237]" />
              <span className="text-sm font-medium text-foreground">False Positives</span>
            </div>
            <div className="text-right">
              <div className="text-base sm:text-lg font-semibold text-[#ffb237]">-8%</div>
              <div className="text-xs text-muted-foreground">vs last week</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function AlertCenter() {
  return (
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
      <div className="flex items-center gap-3 p-5 border-b border-border/20">
        <div className="p-2 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-[14px] text-[#ffb237] border border-[#ffb237]/20">
          <AlertTriangleIcon className="w-4 h-4" />
        </div>
        <h3 className="text-xs font-bold text-[#ffb237] uppercase tracking-[0.2em]">Alert & Notification Center</h3>
      </div>

      <div className="p-5 space-y-3">
        <div className="p-3 bg-red-500/5 border border-red-500/20 rounded-xl">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-start sm:items-center gap-2">
              <div className="w-2 h-2 bg-red-500 rounded-full mt-1 sm:mt-0 flex-shrink-0" />
              <span className="font-medium text-red-400 flex-shrink-0">HIGH</span>
              <span className="text-sm text-muted-foreground">Behavioral Anomaly Policy showing 89% false positives</span>
            </div>
            <span className="text-xs text-muted-foreground self-end sm:self-auto">2m ago</span>
          </div>
        </div>

        <div className="p-3 bg-[#ffb237]/5 border border-[#ffb237]/20 rounded-xl">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-start sm:items-center gap-2">
              <div className="w-2 h-2 bg-[#ffb237] rounded-full mt-1 sm:mt-0 flex-shrink-0" />
              <span className="font-medium text-[#ffb237] flex-shrink-0">MED</span>
              <span className="text-sm text-muted-foreground">Geographic Policy blocked 15 legitimate EU users</span>
            </div>
            <span className="text-xs text-muted-foreground self-end sm:self-auto">5m ago</span>
          </div>
        </div>

        <div className="p-3 bg-[#31d0aa]/5 border border-[#31d0aa]/20 rounded-xl">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-start sm:items-center gap-2">
              <div className="w-2 h-2 bg-[#31d0aa] rounded-full mt-1 sm:mt-0 flex-shrink-0" />
              <span className="font-medium text-[#31d0aa] flex-shrink-0">LOW</span>
              <span className="text-sm text-muted-foreground">Business Hours Policy evaluation time increased to 45ms</span>
            </div>
            <span className="text-xs text-muted-foreground self-end sm:self-auto">12m ago</span>
          </div>
        </div>
      </div>
    </div>
  );
}

export function LoadingState() {
  return (
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
      <div className="p-4 sm:p-6">
        <div className="flex items-center justify-center h-32 sm:h-48">
          <RefreshCwIcon className="h-6 w-6 text-[#1fc7d4] animate-spin" />
          <span className="ml-2 text-muted-foreground">Loading monitoring data...</span>
        </div>
      </div>
    </div>
  );
}
