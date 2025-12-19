/**
 * FRONTEND UNIFIED CARD - Re-export from Shared
 * 
 * This file now re-exports the unified card components from shared.
 * All variants, sections, and legacy aliases are preserved.
 */

export {
  AdminCard,
  AnalyticsCard, EPSXCard, EPSXCardContent,
  EPSXCardFooter, EPSXCardHeader, GlassCard,
  // Legacy aliases
  MetroCard, MetroListCard, MetroStatsCard,
  // Specialized variants
  PancakeCard, PremiumCard, ProfessionalCard, ProfessionalFeatureCard, ProfessionalListCard, ProfessionalStatsCard,
  // Main UnifiedCard
  UnifiedCard, UnifiedCardContent,
  UnifiedCardFooter, UnifiedCardHeader, UnifiedFeatureCard, UnifiedListCard,
  // Specialized cards
  UnifiedStatsCard, type AccentPosition, type UnifiedCardPadding,
  // Types
  type UnifiedCardProps,
  type UnifiedCardSectionProps, type UnifiedCardSize, type UnifiedCardVariant, type UnifiedFeatureCardProps, type UnifiedListCardProps,
  type UnifiedListItem, type UnifiedStatsCardProps
} from '@/shared/components/cards/CardVariants';

// Re-export default
export { UnifiedCard as default } from '@/shared/components/cards/CardVariants';
