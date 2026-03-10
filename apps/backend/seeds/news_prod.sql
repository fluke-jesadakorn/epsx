-- ============================================================
-- EPSX NEWS SEED (production)
-- Adjusted for business-oriented language (no technical jargon)
-- Run: psql -h localhost -p 5432 -U epsx_user -d epsx_prod -f apps/backend/seeds/news_prod.sql
-- ============================================================

INSERT INTO news_articles (
  title, slug, summary, content, cover_image_url,
  author_wallet, status, tags, published_at, is_pinned, pinned_at
) VALUES

-- 1. Analysis Performance (Overwriting technical version)
(
  'Strategic Analysis Performance for Operational Excellence',
  'optimizing-high-throughput-analytics-rust',
  'How EPSX leverages high-performance data processing to deliver precise rankings and insights for business decision-making.',
  E'## Analysis Excellence as a Strategic Priority\n\nFor professional users and serious business participants, the reliability of data insights is a mandatory requirement. In an environment where every data point can represent a significant shift in organizational position, the quality of a platform\'s analysis engine becomes its most critical asset. At EPSX, we recognize that extreme performance is the foundation upon which all successful market strategies are built. This is why we made the strategic decision to design our core analysis framework for uncompromising speed and precision.\n\n## Reliability During High Activity Periods\n\nOne of the primary challenges in data analysis is maintaining platform stability when information volume increases across multiple sectors. Traditional analytical frameworks often struggle with large-scale data management and processing under heavy load, leading to delays or even total unavailability during the most critical market moments. By leveraging a high-efficiency processing model, EPSX eliminates these common performance barriers. Our engine is designed to remain completely responsive even during the highest periods of activity, ensuring that you have uninterrupted access to the intelligence you need most.\n\n## Precision at Scale: The Business Standard\n\nOur high-performance foundation allows us to process millions of complex data signals every second with absolute precision. We don\'t just aggregate data; we analyze it in real-time to surface movements and ranking shifts that others miss. This high-frequency processing ensures that the rankings and metrics you see on your EPSX dashboard are always current, providing a level of data integrity that sets a new standard for professional-grade analytics.\n\n## Long-Term Commitment to Excellence\n\nOur commitment to performance extends beyond just speed. By building a robust, high-precision analysis platform, we have created an environment that is inherently more secure and reliable for our clients. This long-term focus ensures that EPSX remains the premier choice for organizations that require the highest quality data rankings to inform their strategic decisions.\n\n---\n\nFor inquiries regarding our analytical methodology or platform capabilities, please engage with our [Client Relations](/contact) team.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["strategy", "performance"]'::jsonb,
  NOW() - INTERVAL '60 days',
  true,
  NOW() - INTERVAL '60 days'
),

-- 2. Strategic Launch
(
  'Strategic Launch of EPSX: Institutional-Grade Market Insights',
  'strategic-launch-epsx',
  'EPSX is now operational, providing streamlined access to essential market metrics and professional portfolio management.',
  E'## Strategic Operational Launch\n\nEPSX has been established to deliver professional-grade market intelligence through a streamlined, high-performance interface. Our mission is to provide clear, actionable insights without the complexity typically associated with professional tools.\n\nOur platform offers comprehensive market performance tracking, personalized portfolio oversight, and advanced analytics. We prioritize a clean, professional user experience that allows for direct engagement with critical data points.\n\nWhile our roadmap includes significant future enhancements, the current suite of tools is fully operational and optimized for immediate business application.\n\nAccess the [Market Analysis](/analytics) suite to begin your engagement.\n\nWelcome to the EPSX ecosystem.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["announcement", "business"]'::jsonb,
  NOW() - INTERVAL '52 days',
  false,
  NULL
),

-- 3. Performance Metrics
(
  'Proprietary Performance Metrics and Strategic Positioning',
  'performance-metrics-positioning',
  'An overview of our methodology for identifying market momentum and prioritizing growth indicators.',
  E'## Performance Methodology Overview\n\nOur proprietary ranking system evaluates assets through a multi-factor analysis of market activity signals, including volume dynamics and relative performance within specific market sectors.\n\nThis quantitative approach identifies high-momentum opportunities, enabling efficient resource allocation without the need for manual data processing.\n\n## Dynamic Position Updates\n\nRankings are updated continuously to reflect current market conditions. Our system prioritizes assets showing active growth and strategic positioning, ensuring that our insights remain relevant to the current business environment.\n\n## Expanded Insight Access\n\nAdvanced performance data and deeper market rankings are available through our tiered service solutions. Review our [Service Tiers](/plans) to identify the level for your business requirements.\n\nFor inquiries regarding our scoring methodology, please engage with our [Client Relations](/contact) team.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["methodology", "insights"]'::jsonb,
  NOW() - INTERVAL '38 days',
  false,
  NULL
),

-- 4. Service Tier Alignment
(
  'Integrated Service Solutions: Professional Tier Alignment',
  'service-tier-alignment',
  'Our refined service structure is designed to align with various professional requirements and organizational scales.',
  E'## Optimized Service Tier Structure\n\nBased on professional feedback, we have refined our service offerings to ensure better alignment with client needs. Our current structure provides scalable access to the EPSX platform.\n\nOur primary solutions include:\n\n- **Daily Access Pass** — Full platform engagement for a 24-hour period, ideal for initial evaluation.\n- **Essential Suite** — Monthly professional access including standard performance metrics and analytics.\n- **Perpetual License** — A single-commitment solution providing permanent access to all advanced rankings and integration tools.\n- **Enterprise Solution** — Scalable access for organizational teams requiring high-volume throughput and enhanced system limits.\n\nWe also offer specialized **Integration Access** for custom business applications leveraging our data ecosystem.\n\nEach solution provides a foundation for market engagement, with variations based on depth of insight and duration of access.\n\nReview the complete structure on our [Service Tiers](/plans) page.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["solutions", "tier-update"]'::jsonb,
  NOW() - INTERVAL '25 days',
  false,
  NULL
),

-- 5. Portfolio Management
(
  'Enhanced Portfolio Management Solutions',
  'enhanced-portfolio-management',
  'Our asset tracking capabilities have been integrated into a unified portfolio view for streamlined oversight.',
  E'## Strategic Asset Oversight\n\nWe have enhanced our portfolio management capabilities to allow for more efficient oversight of target assets. This solution enables the consolidation of selected market interests into a single, professional interface.\n\n## Implementation Steps\n\n1. Access the analysis suite\n2. Identify target assets for oversight\n3. Select the priority indicator to include in your dashboard\n\nThe asset will be integrated immediately into your [Strategic Portfolio](/portfolio) for ongoing monitoring.\n\n## Ecosystem Integration\n\nYour selected assets are synchronized across your professional profile, ensuring consistent access from any business terminal or mobile interface.\n\n## Dynamic Management\n\nAdjust your monitored assets at any time with a single interaction, maintaining a focused and relevant oversight list without administrative overhead.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["portfolio", "oversight"]'::jsonb,
  NOW() - INTERVAL '14 days',
  false,
  NULL
),

-- 6. Strategic Roadmap
(
  'Strategic Roadmap and Future Capabilities',
  'strategic-roadmap-future',
  'A preview of upcoming system enhancements, including automated alerts and expanded analytical depth.',
  E'## Strategic Development Initiatives\n\nWe maintain a transparent roadmap of our commitment to platform excellence. Current initiatives include:\n\n### Automated Market Alerts\nClients will soon be able to establish specific performance thresholds. When these milestones are achieved, the system will initiate professional communications via prioritized channels.\n\n### Mobile Experience Optimization\nWe are currently refining our mobile engagement interface to ensure seamless professional access while in the field, prioritizing clarity and speed.\n\n### Advanced Analytical Filtering\nOur upcoming enhancements to the analysis suite will allow for deeper segmentation by sector, capitalization, and custom business cycles.\n\n### Proprietary Solutions\nSeveral enterprise-focused capabilities are currently in internal testing. We look forward to announcing these strategic additions as they reach operational readiness.\n\n---\n\nFor feedback or requests regarding our strategic direction, please engage with our [Support](/contact) team. We prioritize professional input in our development process.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["roadmap", "strategy"]'::jsonb,
  NOW() - INTERVAL '5 days',
  false,
  NULL
)

ON CONFLICT (slug) DO UPDATE SET
  title        = EXCLUDED.title,
  summary      = EXCLUDED.summary,
  content      = EXCLUDED.content,
  tags         = EXCLUDED.tags,
  status       = EXCLUDED.status,
  published_at = EXCLUDED.published_at,
  is_pinned    = EXCLUDED.is_pinned,
  pinned_at    = EXCLUDED.pinned_at,
  updated_at   = NOW();

SELECT title, status, published_at, is_pinned FROM news_articles ORDER BY published_at DESC;
