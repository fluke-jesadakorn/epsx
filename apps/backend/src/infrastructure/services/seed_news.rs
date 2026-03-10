//! News Seeder
//!
//! Seeds production-ready news articles on startup with business-oriented language.
//! Uses ON CONFLICT (slug) to safely re-run (idempotent).

use diesel_async::RunQueryDsl;
use tracing::{info, error};

use crate::prelude::TlsPool;

struct NewsDef {
    title: &'static str,
    slug: &'static str,
    summary: &'static str,
    content: &'static str,
    tags: &'static str,
    is_pinned: bool,
    days_ago: i64,
}

const NEWS_ARTICLES: &[NewsDef] = &[
    NewsDef {
        title: "Strategic Analysis Performance for Operational Excellence",
        slug: "optimizing-high-throughput-analytics-rust",
        summary: "How EPSX leverages high-performance data processing to deliver precise rankings and insights for business decision-making.",
        content: "## Analysis Excellence as a Strategic Priority\n\nFor professional users and serious business participants, the reliability of data insights is a mandatory requirement. In an environment where every data point can represent a significant shift in organizational position, the quality of a platform's analysis engine becomes its most critical asset. At EPSX, we recognize that extreme performance is the foundation upon which all successful business strategies are built. This is why we made the strategic decision to design our core analysis framework for uncompromising speed and precision.\n\n## Reliability During High Activity Periods\n\nOne of the primary challenges in data analysis is maintaining platform stability when information volume increases across multiple sectors. Traditional analytical frameworks often struggle with large-scale data management and processing under heavy load, leading to delays or even total unavailability during the most critical market moments. By leveraging a high-efficiency processing model, EPSX eliminates these common performance barriers. Our engine is designed to remain completely responsive even during the highest periods of activity, ensuring that you have uninterrupted access to the intelligence you need most.\n\n## Precision at Scale: The Business Standard\n\nOur high-performance foundation allows us to process millions of complex data signals every second with absolute precision. We don't just aggregate data; we analyze it in real-time to surface movements and ranking shifts that others miss. This high-frequency processing ensures that the rankings and metrics you see on your EPSX dashboard are always current, providing a level of data integrity that sets a new standard for professional-grade analytics.\n\n## Long-Term Commitment to Excellence\n\nOur commitment to performance extends beyond just speed. By building a robust, high-precision analysis platform, we have created an environment that is inherently more secure and reliable for our clients. This long-term focus ensures that EPSX remains the premier choice for organizations that require the highest quality data rankings to inform their strategic decisions.\n\n---\n\nFor inquiries regarding our analytical methodology or platform capabilities, please engage with our [Client Relations](/contact) team.",
        tags: "[\"strategy\", \"performance\"]",
        is_pinned: true,
        days_ago: 60,
    },
    NewsDef {
        title: "Strategic Launch of EPSX: Institutional-Grade Market Insights",
        slug: "strategic-launch-epsx",
        summary: "EPSX is now operational, providing streamlined access to essential market metrics and professional portfolio management.",
        content: "## Strategic Operational Launch\n\nEPSX has been established to deliver professional-grade market intelligence through a streamlined, high-performance interface. Our mission is to provide clear, actionable insights without the complexity typically associated with professional tools.\n\nOur platform offers comprehensive market performance tracking, personalized portfolio oversight, and advanced analytics. We prioritize a clean, professional user experience that allows for direct engagement with critical data points.\n\nWhile our roadmap includes significant future enhancements, the current suite of tools is fully operational and optimized for immediate business application.\n\nAccess the [Market Analysis](/analytics) suite to begin your engagement.\n\nWelcome to the EPSX ecosystem.",
        tags: "[\"announcement\", \"business\"]",
        is_pinned: false,
        days_ago: 52,
    },
    NewsDef {
        title: "Proprietary Performance Metrics and Strategic Positioning",
        slug: "performance-metrics-positioning",
        summary: "An overview of our methodology for identifying market momentum and prioritizing growth indicators.",
        content: "## Performance Methodology Overview\n\nOur proprietary ranking system evaluates assets through a multi-factor analysis of market activity signals, including volume dynamics and relative performance within specific market sectors.\n\nThis quantitative approach identifies high-momentum opportunities, enabling efficient resource allocation without the need for manual data processing.\n\n## Dynamic Position Updates\n\nRankings are updated continuously to reflect current market conditions. Our system prioritizes assets showing active growth and strategic positioning, ensuring that our insights remain relevant to the current business environment.\n\n## Expanded Insight Access\n\nAdvanced performance data and deeper market rankings are available through our tiered service solutions. Review our [Service Tiers](/plans) to identify the optimal level for your business requirements.\n\nFor inquiries regarding our scoring methodology, please engage with our [Client Relations](/contact) team.",
        tags: "[\"methodology\", \"insights\"]",
        is_pinned: false,
        days_ago: 38,
    },
    NewsDef {
        title: "Integrated Service Solutions: Professional Tier Alignment",
        slug: "service-tier-alignment",
        summary: "Our refined service structure is designed to align with various professional requirements and organizational scales.",
        content: "## Optimized Service Tier Structure\n\nBased on professional feedback, we have refined our service offerings to ensure better alignment with client needs. Our current structure provides scalable access to the EPSX platform.\n\nOur primary solutions include:\n\n- **Daily Access Pass** — Full platform engagement for a 24-hour period, ideal for initial evaluation.\n- **Essential Suite** — Monthly professional access including standard performance metrics and analytics.\n- **Perpetual License** — A single-commitment solution providing permanent access to all advanced rankings and integration tools.\n- **Enterprise Solution** — Scalable access for organizational teams requiring high-volume throughput and enhanced system limits.\n\nWe also offer specialized **Integration Access** for custom business applications leveraging our data ecosystem.\n\nEach solution provides a foundation for market engagement, with variations based on depth of insight and duration of access.\n\nReview the complete structure on our [Service Tiers](/plans) page.",
        tags: "[\"solutions\", \"tier-update\"]",
        is_pinned: false,
        days_ago: 25,
    },
    NewsDef {
        title: "Enhanced Portfolio Management Solutions",
        slug: "enhanced-portfolio-management",
        summary: "Our asset tracking capabilities have been integrated into a unified portfolio view for streamlined oversight.",
        content: "## Strategic Asset Oversight\n\nWe have enhanced our portfolio management capabilities to allow for more efficient oversight of target assets. This solution enables the consolidation of selected market interests into a single, professional interface.\n\n## Implementation Steps\n\n1. Access the analysis suite\n2. Identify target assets for oversight\n3. Select the priority indicator to include in your dashboard\n\nThe asset will be integrated immediately into your [Strategic Portfolio](/portfolio) for ongoing monitoring.\n\n## Ecosystem Integration\n\nYour selected assets are synchronized across your professional profile, ensuring consistent access from any business terminal or mobile interface.\n\n## Dynamic Management\n\nAdjust your monitored assets at any time with a single interaction, maintaining a focused and relevant oversight list without administrative overhead.",
        tags: "[\"portfolio\", \"oversight\"]",
        is_pinned: false,
        days_ago: 14,
    },
    NewsDef {
        title: "Strategic Roadmap and Future Capabilities",
        slug: "strategic-roadmap-future",
        summary: "A preview of upcoming system enhancements, including automated alerts and expanded analytical depth.",
        content: "## Strategic Development Initiatives\n\nWe maintain a transparent roadmap of our commitment to platform excellence. Current initiatives include:\n\n### Automated Market Alerts\nClients will soon be able to establish specific performance thresholds. When these milestones are achieved, the system will initiate professional communications via prioritized channels.\n\n### Mobile Experience Optimization\nWe are currently refining our mobile engagement interface to ensure seamless professional access while in the field, prioritizing clarity and speed.\n\n### Advanced Analytical Filtering\nOur upcoming enhancements to the analysis suite will allow for deeper segmentation by sector, capitalization, and custom business cycles.\n\n### Proprietary Solutions\nSeveral enterprise-focused capabilities are currently in internal testing. We look forward to announcing these strategic additions as they reach operational readiness.\n\n---\n\nFor feedback or requests regarding our strategic direction, please engage with our [Support](/contact) team. We prioritize professional input in our development process.",
        tags: "[\"roadmap\", \"strategy\"]",
        is_pinned: false,
        days_ago: 5,
    },
];

/// Seed production news articles into the database.
/// Safe to call multiple times (idempotent via ON CONFLICT on slug).
pub async fn seed_production_news(pool: &TlsPool) {
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get DB connection for news seeding: {}", e);
            return;
        }
    };

    for def in NEWS_ARTICLES {
        let published_at_sql = format!("NOW() - INTERVAL '{} days'", def.days_ago);
        let pinned_at_sql = if def.is_pinned { "NOW() - INTERVAL '{} days'" } else { "NULL" };
        let pinned_at_val = pinned_at_sql.replace("{}", &def.days_ago.to_string());

        let result = diesel::sql_query(
            format!(r#"INSERT INTO news_articles (
                title, slug, summary, content, cover_image_url,
                author_wallet, status, tags, published_at, is_pinned, pinned_at
            ) VALUES (
                $1, $2, $3, $4, NULL,
                '0x0000000000000000000000000000000000000000', 'published', $5::jsonb, {}, $6, {}
            )
            ON CONFLICT (slug) DO UPDATE SET
                title = EXCLUDED.title,
                summary = EXCLUDED.summary,
                content = EXCLUDED.content,
                tags = EXCLUDED.tags,
                status = EXCLUDED.status,
                published_at = EXCLUDED.published_at,
                is_pinned = EXCLUDED.is_pinned,
                pinned_at = EXCLUDED.pinned_at,
                updated_at = NOW()"#, 
                published_at_sql.replace("{}", &def.days_ago.to_string()),
                pinned_at_val
            )
        )
        .bind::<diesel::sql_types::Text, _>(def.title)
        .bind::<diesel::sql_types::Text, _>(def.slug)
        .bind::<diesel::sql_types::Text, _>(def.summary)
        .bind::<diesel::sql_types::Text, _>(def.content)
        .bind::<diesel::sql_types::Text, _>(def.tags)
        .bind::<diesel::sql_types::Bool, _>(def.is_pinned)
        .execute(&mut conn)
        .await;

        if let Err(e) = result {
            error!("Failed to seed news article {}: {}", def.title, e);
            continue;
        }

        info!("Seeded news article: {} ({})", def.title, def.slug);
    }

    info!("Production news seeding complete");
}
