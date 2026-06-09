//! Built-out pages from the original Next.js frontend.
//!
//! Each `*_body()` function returns a `String` of HTML for that page's
//! `<main>` region. The BFF's `render_page()` in `main.rs` wires these
//! into the design-system shell, navbar, and footer.

use epsx_templates::components::{BadgeKind, Input, StatCard, Tabs};

// =====================================================================
// /news — News index (matches epsx.io /news page exactly)
// =====================================================================

pub fn news_index_body() -> String {
    let articles = vec![
        ("/news/strategic-roadmap-future", "roadmap", "strategy", "Strategic Roadmap and Future Capabilities", "A preview of upcoming system enhancements, including automated alerts and expanded analytical depth.", "May 9, 2026"),
        ("/news/enhanced-portfolio-management", "portfolio", "oversight", "Enhanced Portfolio Management Solutions", "Our asset tracking capabilities have been integrated into a unified portfolio view for streamlined oversight.", "April 30, 2026"),
        ("/news/integrated-service-solutions", "solutions", "tier-update", "Integrated Service Solutions: Professional Tier Alignment", "Our refined service structure is designed to align with various professional requirements and organizational scales.", "April 19, 2026"),
        ("/news/proprietary-performance-metrics", "methodology", "insights", "Proprietary Performance Metrics and Strategic Positioning", "An overview of our methodology for identifying market momentum and prioritizing growth indicators.", "April 6, 2026"),
        ("/news/strategic-launch-epsx", "announcement", "business", "Strategic Launch of EPSX: Institutional-Grade Market Insights", "EPSX is now operational, providing streamlined access to essential market metrics and professional portfolio management.", "March 23, 2026"),
        ("/news/optimizing-high-throughput-analytics-rust", "strategy", "performance", "Strategic Analysis Performance for Operational Excellence", "How EPSX leverages high-performance data processing to deliver precise rankings and insights for business decision-making.", "March 15, 2026"),
        ("/news/real-time-intelligence", "redis", "real-time", "Real-Time Intelligence: Capturing Market Opportunities as They Happen", "The EPSX dashboard removes the gap between on-chain events and your decision-making. Learn how our Redis-powered real-time data pipeline keeps you at the absolute forefront of the market.", "March 10, 2026"),
        ("/news/securing-the-future", "web3", "security", "Securing the Future: Enterprise-Grade Trust in a Web3 World", "EPSX leads the industry in user privacy and security by adopting wallet-first authentication. Learn how our implementation of SIWE protects your assets and identity.", "March 10, 2026"),
        ("/news/scalable-foundation", "postgresql", "database", "Built for Ambition: A Scalable Foundation for Global Analytics", "Scaling a global analytics platform requires an industrial-strength architecture. Discover how EPSX manages billions of time-series records with ease.", "March 10, 2026"),
        ("/news/smarter-decisions-ai", "ai", "machine-learning", "Smarter Decisions: How EPSX AI Navigates Market Complexity", "Leverage the power of artificial intelligence to filter out noise and surface high-signal trends. Discover how the unique EPSX Sentiment Score provides actionable market intelligence.", "March 10, 2026"),
    ];
    let cards = articles.iter().map(|(href, tag1, tag2, title, excerpt, date)| {
        format!(
            r##"<a href="{href}" style="text-decoration:none;color:inherit;display:block;" class="block group">
      <article class="card-glass hover-scale" style="padding:0;overflow:hidden;display:flex;flex-direction:column;height:100%;">
        <div style="width:100%;height:12rem;background:linear-gradient(135deg, rgba(168,85,247,0.15), rgba(31,199,212,0.05));display:flex;align-items:center;justify-content:center;">
          <span style="font-size:0.75rem;font-weight:600;color:rgba(168,85,247,0.6);text-transform:uppercase;letter-spacing:0.05em;">{tag1}</span>
        </div>
        <div style="padding:1.25rem;display:flex;flex-direction:column;flex:1;">
          <div style="display:flex;gap:0.375rem;margin-bottom:0.75rem;flex-wrap:wrap;">
            <span class="news-tag" style="font-size:0.6875rem;">{tag1}</span>
            <span class="news-tag" style="font-size:0.6875rem;">{tag2}</span>
          </div>
          <h3 style="font-size:1.0625rem;font-weight:700;margin-bottom:0.5rem;line-height:1.3;color:var(--text);">{title}</h3>
          <p style="color:var(--text-muted);font-size:0.875rem;line-height:1.5;flex:1;">{excerpt}</p>
          <div style="margin-top:1rem;padding-top:1rem;border-top:1px solid var(--border);font-size:0.8125rem;color:var(--text-subtle);display:flex;justify-content:space-between;align-items:center;">
            <span>{date}</span>
            <span style="color:var(--epsx-cyan);font-weight:500;">Read &rarr;</span>
          </div>
        </div>
      </article>
    </a>"##,
            href = href, tag1 = tag1, tag2 = tag2, title = title, excerpt = excerpt, date = date
        )
    }).collect::<Vec<_>>().join("\n");

    let cards_html = format!(
        r##"<div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(300px, 1fr));gap:1.5rem;">{cards}</div>"##,
        cards = cards
    );

    format!(
        r##"<section class="section">
<div class="container-x">
  <div style="text-align:center;margin-bottom:2.5rem;">
    <div style="display:flex;align-items:center;justify-content:center;gap:0.5rem;margin-bottom:1rem;">
      <i data-lucide="newspaper" style="color:var(--epsx-cyan);width:1.5rem;height:1.5rem;"></i>
      <span style="font-size:0.875rem;color:var(--epsx-cyan);font-weight:500;text-transform:uppercase;letter-spacing:0.05em;">EPSX Platform</span>
    </div>
    <h1 style="font-size:2.5rem;font-weight:800;margin:0.5rem 0 1rem;color:var(--text);">News &amp; Updates</h1>
    <p style="font-size:1rem;color:var(--text-muted);max-width:42rem;margin:0 auto;">Stay informed with the latest platform updates, feature releases, and market insights from the EPSX team.</p>
    <div style="margin-top:1rem;font-size:0.875rem;color:var(--text-subtle);">10 articles</div>
  </div>
  {cards_html}
</div>
</section>"##,
        cards_html = cards_html
    )
}

pub fn news_post_body(slug: &str) -> String {
    format!(
        r##"<section class="section">
<div class="container-x" style="max-width:54rem;">
  <a href="/news" class="nav-link" style="margin-bottom:1.5rem;display:inline-flex;">
    <i class="fa-solid fa-arrow-left"></i> Back to news
  </a>
  <span class="badge badge-primary" style="margin-bottom:0.75rem;">Announcement</span>
  <h1 style="font-size:2.75rem;font-weight:800;margin-bottom:1rem;line-height:1.2;">{}</h1>
  <div style="display:flex;gap:1.5rem;color:var(--text-muted);font-size:0.875rem;margin-bottom:2.5rem;padding-bottom:1.5rem;border-bottom:1px solid var(--border);">
    <span><i class="fa-regular fa-calendar"></i> June 9, 2026</span>
    <span><i class="fa-regular fa-clock"></i> 5 min read</span>
    <span><i class="fa-regular fa-user"></i> EPSX Team</span>
  </div>
  <article class="card-insight" style="font-size:1.0625rem;line-height:1.8;color:var(--text-muted);">
    <p style="color:var(--text);font-size:1.25rem;line-height:1.6;margin-bottom:1.5rem;font-weight:500;">EPSX is now live on BNB Smart Chain mainnet. Builders, creators, and merchants can now leverage on-chain analytics, stablecoin payments, and programmable subscription vaults.</p>
    <h2 style="font-size:1.75rem;font-weight:700;margin:2rem 0 1rem;color:var(--text);">Why BSC?</h2>
    <p>BSC offers sub-second finality, predictable gas costs under $0.01 per transaction, and the deepest stablecoin liquidity in crypto. EPSX is purpose-built to take advantage of all three.</p>
    <h2 style="font-size:1.75rem;font-weight:700;margin:2rem 0 1rem;color:var(--text);">What's included at launch</h2>
    <ul style="list-style:none;padding:0;display:grid;gap:0.75rem;">
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i class="fa-solid fa-check" style="color:var(--epsx-green);margin-top:0.4rem;"></i><div><strong>Real-time analytics</strong> &mdash; on-chain rankings updated every block.</div></li>
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i class="fa-solid fa-check" style="color:var(--epsx-green);margin-top:0.4rem;"></i><div><strong>Subscription vaults</strong> &mdash; per-merchant isolated risk, USDC/USDT payouts.</div></li>
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i class="fa-solid fa-check" style="color:var(--epsx-green);margin-top:0.4rem;"></i><div><strong>Paymaster</strong> &mdash; sponsor gas, charge users in stablecoins via ERC-4337.</div></li>
      <li style="display:flex;gap:0.75rem;align-items:flex-start;"><i class="fa-solid fa-check" style="color:var(--epsx-green);margin-top:0.4rem;"></i><div><strong>Visual builder</strong> &mdash; drag-and-drop page editor with web3-native auth.</div></li>
    </ul>
    <p style="margin-top:2rem;">Read the full announcement on our blog, or dive into the technical documentation.</p>
  </article>
  <div style="display:flex;gap:1rem;margin-top:2.5rem;flex-wrap:wrap;">
    <a href="/docs" class="btn btn-outline"><i class="fa-solid fa-book"></i> Read the docs</a>
    <a href="/auth" class="btn btn-gradient"><i class="fa-solid fa-rocket"></i> Get started</a>
  </div>
</div>
</section>"##,
        slug
    )
}

// =====================================================================
// /plans — Pricing plans grid
// =====================================================================

pub fn plans_body() -> String {
    let free = r##"<div class="card-insight" style="padding:2rem;display:flex;flex-direction:column;">
      <h3 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Free</h3>
      <div style="font-size:2.5rem;font-weight:800;margin-bottom:0.5rem;">$0<span style="font-size:1rem;font-weight:500;color:var(--text-muted);">/month</span></div>
      <p style="color:var(--text-muted);font-size:0.875rem;margin-bottom:1.5rem;">For curious users exploring EPSX.</p>
      <ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.625rem;flex:1;">
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> 100 API calls/day</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Basic analytics</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Community support</li>
        <li style="display:flex;gap:0.5rem;align-items:center;color:var(--text-subtle);"><i class="fa-solid fa-xmark"></i> Custom webhooks</li>
      </ul>
      <a href="/auth" class="btn btn-outline btn-block">Get Started</a>
    </div>"##;

    let pro = r##"<div class="card-insight" style="padding:2rem;display:flex;flex-direction:column;border:2px solid var(--epsx-orange);position:relative;">
      <span class="badge badge-primary" style="position:absolute;top:-0.75rem;left:50%;transform:translateX(-50%);box-shadow:var(--shadow-orange);">
        <i class="fa-solid fa-star"></i> POPULAR
      </span>
      <h3 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Pro</h3>
      <div class="gradient-text" style="font-size:2.5rem;font-weight:800;margin-bottom:0.5rem;">$29<span style="font-size:1rem;font-weight:500;color:var(--text-muted);-webkit-text-fill-color:var(--text-muted);">/month</span></div>
      <p style="color:var(--text-muted);font-size:0.875rem;margin-bottom:1.5rem;">For active traders and developers.</p>
      <ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.625rem;flex:1;">
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> 10,000 API calls/day</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Advanced analytics</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Priority support</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Custom webhooks</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Watchlist + alerts</li>
      </ul>
      <a href="/payment?plan=pro" class="btn btn-gradient btn-block">Subscribe</a>
    </div>"##;

    let enterprise = r##"<div class="card-insight" style="padding:2rem;display:flex;flex-direction:column;">
      <h3 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Enterprise</h3>
      <div style="font-size:2.5rem;font-weight:800;margin-bottom:0.5rem;">Custom</div>
      <p style="color:var(--text-muted);font-size:0.875rem;margin-bottom:1.5rem;">For teams and large-scale integrations.</p>
      <ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.625rem;flex:1;">
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Unlimited API calls</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Custom integrations</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Dedicated support</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> SLA guarantee</li>
        <li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> On-prem deployment</li>
      </ul>
      <a href="/contact" class="btn btn-outline btn-block">Contact Us</a>
    </div>"##;

    let faq = r##"<div class="card-insight" style="margin-bottom:1rem;">
      <h3 style="font-size:1.125rem;font-weight:700;margin-bottom:0.5rem;">Can I switch plans at any time?</h3>
      <p style="color:var(--text-muted);font-size:0.9375rem;line-height:1.6;">Yes. You can upgrade or downgrade your plan at any time. Upgrades take effect immediately, and downgrades apply at the end of your current billing cycle.</p>
    </div>
    <div class="card-insight" style="margin-bottom:1rem;">
      <h3 style="font-size:1.125rem;font-weight:700;margin-bottom:0.5rem;">What payment methods do you accept?</h3>
      <p style="color:var(--text-muted);font-size:0.9375rem;line-height:1.6;">We accept USDT and USDC on BSC, plus all major credit cards via our payment processor. Enterprise customers can pay by invoice in USD.</p>
    </div>
    <div class="card-insight" style="margin-bottom:1rem;">
      <h3 style="font-size:1.125rem;font-weight:700;margin-bottom:0.5rem;">Do you offer a free trial?</h3>
      <p style="color:var(--text-muted);font-size:0.9375rem;line-height:1.6;">The Free tier is always available &mdash; no credit card required. You can also reach out to sales for an extended trial of Pro or Enterprise features.</p>
    </div>
    <div class="card-insight">
      <h3 style="font-size:1.125rem;font-weight:700;margin-bottom:0.5rem;">What happens if I exceed my plan limits?</h3>
      <p style="color:var(--text-muted);font-size:0.9375rem;line-height:1.6;">We will notify you when you approach your limit. API calls over your quota return a 429 response until the next cycle, or you can upgrade to a higher plan for instant access.</p>
    </div>"##;

    format!(
        r##"<section class="section">
<div class="container-x" style="text-align:center;">
  <span class="badge-pill"><i class="fa-solid fa-tag" style="color:var(--epsx-orange);"></i> Plans</span>
  <h1 class="gradient-text" style="font-size:3rem;font-weight:800;margin:1rem 0 1rem;">Choose Your EPSX Plan</h1>
  <p style="font-size:1.125rem;color:var(--text-muted);max-width:42rem;margin:0 auto 3rem;">Start free, scale as you grow. No hidden fees, ever.</p>

  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(280px, 1fr));gap:1.5rem;max-width:72rem;margin:0 auto;text-align:left;">
    {free}
    {pro}
    {enterprise}
  </div>
</div>
</section>

<section class="section" style="background:var(--bg-secondary);">
<div class="container-x" style="max-width:48rem;">
  <div style="text-align:center;margin-bottom:3rem;">
    <span class="badge-pill"><i class="fa-solid fa-circle-question" style="color:var(--epsx-orange);"></i> FAQ</span>
    <h2 style="font-size:2.25rem;font-weight:800;margin:1rem 0;">Frequently asked questions</h2>
  </div>
  {faq}
</div>
</section>"##,
        free = free,
        pro = pro,
        enterprise = enterprise,
        faq = faq,
    )
}

// =====================================================================
// /portfolio — Watchlist / portfolio page
// =====================================================================

pub fn portfolio_body() -> String {
    // Matches epsx.io's actual /portfolio page
    r##"<section class="section">
<div class="container-x" style="max-width:64rem;">
  <div style="text-align:center;margin-bottom:3rem;">
    <h1 style="font-size:2.5rem;font-weight:800;margin:0 0 0.75rem;color:var(--text);">Portfolio</h1>
    <p style="color:var(--text-muted);font-size:1rem;margin:0;">Track your watchlisted stocks</p>
    <div style="margin-top:1rem;display:inline-flex;align-items:center;gap:0.5rem;padding:0.25rem 0.75rem;border-radius:9999px;background:rgba(16,185,129,0.15);color:#10b981;font-size:0.75rem;font-weight:600;">
      <span style="width:0.5rem;height:0.5rem;border-radius:9999px;background:#10b981;animation:pulse 2s infinite;"></span>
      Live
    </div>
  </div>

  <div class="card-glass" style="padding:2.5rem 2rem;text-align:center;margin-bottom:2rem;border:1px solid var(--border);">
    <div style="display:inline-flex;align-items:center;gap:0.5rem;padding:0.375rem 0.875rem;border-radius:9999px;background:linear-gradient(90deg,rgba(168,85,247,0.15) 0%,rgba(236,72,153,0.15) 100%);border:1px solid rgba(168,85,247,0.3);color:#a855f7;font-size:0.75rem;font-weight:600;margin-bottom:1.5rem;">
      <i data-lucide="lock" style="width:0.875rem;height:0.875rem;"></i>
      Unlock Full Analytics Access
    </div>
    <h2 style="font-size:1.5rem;font-weight:700;margin-bottom:1.5rem;color:var(--text);">Get access to all rankings and premium features</h2>
    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;max-width:36rem;margin:0 auto 2rem;">
      <div style="display:flex;align-items:center;gap:0.5rem;justify-content:center;color:var(--text-muted);font-size:0.875rem;"><i data-lucide="check" style="width:1rem;height:1rem;color:#10b981;"></i> Top 100 stock rankings</div>
      <div style="display:flex;align-items:center;gap:0.5rem;justify-content:center;color:var(--text-muted);font-size:0.875rem;"><i data-lucide="check" style="width:1rem;height:1rem;color:#10b981;"></i> Real-time EPS data</div>
      <div style="display:flex;align-items:center;gap:0.5rem;justify-content:center;color:var(--text-muted);font-size:0.875rem;"><i data-lucide="check" style="width:1rem;height:1rem;color:#10b981;"></i> AI-powered insights</div>
    </div>
    <a href="/auth" class="btn btn-gradient btn-lg"><i data-lucide="log-in" style="width:1rem;height:1rem;"></i> Sign In Free</a>
  </div>

  <div class="card-glass" style="padding:2rem;text-align:center;border:1px solid var(--border);">
    <div style="width:4rem;height:4rem;border-radius:9999px;background:rgba(168,85,247,0.1);display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;">
      <i data-lucide="lock" style="width:1.5rem;height:1.5rem;color:#a855f7;"></i>
    </div>
    <h3 style="font-size:1.25rem;font-weight:700;margin-bottom:0.5rem;color:var(--text);">🔐 Sign In Required</h3>
    <p style="color:var(--text-muted);font-size:0.875rem;margin-bottom:1.5rem;">To view your portfolio, you need basic authentication.</p>
    <div style="display:flex;gap:0.75rem;justify-content:center;flex-wrap:wrap;">
      <a href="/auth" class="btn btn-gradient"><i data-lucide="log-in" style="width:1rem;height:1rem;"></i> Sign In</a>
      <a href="/developer/docs" class="btn btn-outline"><i data-lucide="book-open" style="width:1rem;height:1rem;"></i> Learn More</a>
    </div>
  </div>

  <div style="margin-top:2rem;padding:1.25rem;border-radius:0.75rem;background:rgba(6,182,212,0.05);border:1px solid rgba(6,182,212,0.2);text-align:center;">
    <p style="margin:0;color:var(--text-muted);font-size:0.875rem;">Need help? Check our <a href="/developer/docs" style="color:var(--epsx-cyan);">support documentation</a> or <a href="/contact" style="color:var(--epsx-cyan);">contact support</a>.</p>
  </div>
</div>
</section>"##.to_string()
}

// =====================================================================
// /analytics — Analytics dashboard (mock data)
// =====================================================================

pub fn analytics_body() -> String {
    let header = r##"<div style="display:flex;justify-content:space-between;align-items:flex-end;flex-wrap:wrap;gap:1rem;margin-bottom:2rem;">
    <div>
      <div style="display:flex;align-items:center;gap:1rem;margin-bottom:0.5rem;">
        <div style="width:3rem;height:3rem;border-radius:0.75rem;background:var(--gradient-purple);display:flex;align-items:center;justify-content:center;box-shadow:0 10px 15px -3px rgba(168,85,247,0.25);">
          <i class="fa-solid fa-chart-column" style="color:white;font-size:1.125rem;"></i>
        </div>
        <h1 style="font-size:2.5rem;font-weight:800;margin:0;">Analytics</h1>
        <span class="badge badge-success" style="border-radius:9999px;"><i class="fa-solid fa-circle" style="font-size:0.5em;"></i> Live</span>
        <span class="badge badge-brand" style="border-radius:9999px;"><i class="fa-solid fa-wand-magic-sparkles"></i> AI-Powered</span>
      </div>
      <p style="color:var(--text-muted);font-size:1.125rem;margin:0;">Top-performing assets ranked by EPS growth.</p>
    </div>
  </div>"##;

    let filter_bar = r##"<div class="card-insight" style="padding:1rem;margin-bottom:1.5rem;display:flex;gap:1rem;align-items:center;flex-wrap:wrap;">
    <select class="input" style="width:auto;min-width:160px;">
      <option>All countries</option>
      <option>US</option>
      <option>EU</option>
      <option>UK</option>
    </select>
    <select class="input" style="width:auto;min-width:160px;">
      <option>All sectors</option>
      <option>Technology</option>
      <option>Healthcare</option>
      <option>Finance</option>
    </select>
    <select class="input" style="width:auto;min-width:160px;">
      <option>Sort: EPS growth</option>
      <option>Sort: Rank</option>
      <option>Sort: Volume</option>
    </select>
    <button class="btn btn-gradient btn-sm"><i class="fa-solid fa-filter"></i> Apply</button>
    <button class="btn btn-ghost btn-sm"><i class="fa-solid fa-rotate-left"></i> Reset</button>
  </div>"##;

    let card = |rank: u32, sym: &str, name: &str, price: &str, growth: &str, days: u32, premium: bool| {
        let bg = if premium { "var(--gradient-warm)" } else { "var(--gradient-cool)" };
        format!(
            r##"<article class="card-glass hover-scale" style="padding:1.5rem;cursor:pointer;">
      <div style="display:flex;align-items:flex-start;justify-content:space-between;margin-bottom:0.75rem;">
        <div style="display:flex;gap:0.75rem;align-items:center;">
          <div style="width:2.25rem;height:2.25rem;border-radius:0.5rem;background:{bg};display:flex;align-items:center;justify-content:center;color:white;font-weight:700;font-size:0.75rem;">#{rank}</div>
          <div>
            <div style="font-weight:700;">{sym}</div>
            <div style="font-size:0.75rem;color:var(--text-muted);">{name}</div>
          </div>
        </div>
        <button class="nav-link" style="padding:0.25rem;color:var(--text-subtle);" title="Add to watchlist">
          <i class="fa-regular fa-heart"></i>
        </button>
      </div>
      <div style="display:flex;justify-content:space-between;align-items:flex-end;">
        <div>
          <div style="font-size:1.25rem;font-weight:700;">{price}</div>
          <div style="font-size:0.75rem;color:var(--text-muted);">{days}d to next</div>
        </div>
        <span class="badge badge-success" style="border-radius:9999px;">{growth}</span>
      </div>
    </article>"##,
            rank = rank, sym = sym, name = name, price = price, growth = growth, days = days, bg = bg
        )
    };

    let cards = [
        card(1, "NVDA", "NVIDIA Corp", "$924.79", "+24.8%", 12, true),
        card(2, "MSFT", "Microsoft", "$415.50", "+18.2%", 8, true),
        card(3, "AAPL", "Apple Inc.", "$182.45", "+12.4%", 21, true),
        card(4, "GOOGL", "Alphabet", "$175.20", "+11.7%", 15, true),
        card(5, "AMZN", "Amazon", "$184.30", "+9.8%", 6, true),
        card(6, "META", "Meta Platforms", "$498.75", "+8.5%", 19, false),
    ].join("");

    let pagination = r##"<div style="display:flex;justify-content:center;align-items:center;gap:1rem;margin-top:2rem;">
    <button class="btn btn-outline btn-sm" disabled><i class="fa-solid fa-chevron-left"></i> Previous</button>
    <span style="color:var(--text-muted);font-size:0.875rem;">Page 1 of 24</span>
    <button class="btn btn-outline btn-sm">Next <i class="fa-solid fa-chevron-right"></i></button>
  </div>"##;

    format!(
        r##"<section class="section">
<div class="container-x">
  {header}
  {filter_bar}
  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(260px, 1fr));gap:1.5rem;">
    {cards}
  </div>
  {pagination}
</div>
</section>"##,
        header = header,
        filter_bar = filter_bar,
        cards = cards,
        pagination = pagination,
    )
}

// =====================================================================
// /permissions — User's permissions page
// =====================================================================

pub fn permissions_body() -> String {
    let tabs = Tabs::new("perms")
        .tab("active", "Active")
        .tab("expiring", "Expiring Soon")
        .tab("expired", "Expired")
        .tab("all", "All")
        .tab("analytics", "Analytics")
        .tab("history", "History")
        .active("active")
        .render();

    let total = StatCard::new("Total", "12").icon("key", "var(--epsx-blue-start)").render();
    let active = StatCard::new("Active", "10").icon("circle-check", "var(--epsx-green)").render();
    let expiring = StatCard::new("Expiring", "2").icon("clock", "var(--epsx-amber)").render();
    let expired = StatCard::new("Expired", "0").icon("circle-xmark", "var(--epsx-red)").render();

    let perm_row = |icon: &str, color: &str, platform: &str, title: &str, key: &str, expiry: &str, status: &str| {
        format!(
            r##"<div class="card-insight" style="display:flex;align-items:center;gap:1rem;padding:1rem 1.25rem;">
        <div style="width:2.5rem;height:2.5rem;border-radius:0.5rem;background:rgba(59,130,246,0.1);color:{color};display:flex;align-items:center;justify-content:center;flex-shrink:0;">
          <i class="fa-solid fa-{icon}"></i>
        </div>
        <div style="flex:1;min-width:0;">
          <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.25rem;">
            <span class="badge badge-info" style="font-size:0.6875rem;">{platform}</span>
            <span style="font-weight:600;">{title}</span>
          </div>
          <code style="font-size:0.75rem;color:var(--text-muted);font-family:monospace;">{key}</code>
        </div>
        <div style="text-align:right;flex-shrink:0;">
          <div style="font-size:0.75rem;color:var(--text-muted);">{expiry}</div>
          <span class="badge badge-success" style="margin-top:0.25rem;">{status}</span>
        </div>
      </div>"##,
            icon = icon, color = color, platform = platform, title = title, key = key, expiry = expiry, status = status
        )
    };

    let rows = [
        perm_row("chart-line", "var(--epsx-blue-start)", "EPSX", "Analytics:read", "epsx:analytics:read", "Expires in 28 days", "Active"),
        perm_row("wallet", "var(--epsx-orange)", "EPSX", "Wallet:read", "epsx:wallet:read", "Expires in 28 days", "Active"),
        perm_row("vault", "var(--epsx-purple)", "EPSX", "Subscription:write", "epsx:subscription:write", "Expires in 28 days", "Active"),
        perm_row("key", "var(--epsx-cyan)", "EPSX-API", "Api:read", "epsx:api:read", "Expires in 5 days", "Expiring"),
    ].join("");

    format!(
        r##"<section class="section">
<div class="container-x" style="max-width:64rem;">
  <div style="display:flex;justify-content:space-between;align-items:flex-end;flex-wrap:wrap;gap:1rem;margin-bottom:2rem;">
    <div>
      <span class="badge-pill"><i class="fa-solid fa-key" style="color:var(--epsx-orange);"></i> Permissions</span>
      <h1 style="font-size:2.5rem;font-weight:800;margin:1rem 0 0.5rem;">My Permissions</h1>
      <p style="color:var(--text-muted);">Active and historical permissions across all your plans.</p>
    </div>
    <div style="display:flex;gap:0.5rem;">
      <button class="btn btn-outline btn-sm"><i class="fa-solid fa-download"></i> Export</button>
      <button class="btn btn-gradient btn-sm"><i class="fa-solid fa-rotate"></i> Refresh</button>
    </div>
  </div>

  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(180px, 1fr));gap:1rem;margin-bottom:2rem;">
    {total}
    {active}
    {expiring}
    {expired}
  </div>

  {tabs}

  <div style="margin-top:1.5rem;display:grid;gap:0.75rem;">
    {rows}
  </div>
</div>
</section>"##,
        total = total,
        active = active,
        expiring = expiring,
        expired = expired,
        tabs = tabs,
        rows = rows,
    )
}

// =====================================================================
// /chat — Support chat inbox
// =====================================================================

pub fn chat_body() -> String {
    let sidebar_header = r##"<div style="padding:1.5rem;border-bottom:1px solid var(--border);">
    <div style="display:flex;align-items:center;gap:0.75rem;margin-bottom:1rem;">
      <div style="width:2.5rem;height:2.5rem;border-radius:0.625rem;background:var(--gradient-brand);display:flex;align-items:center;justify-content:center;box-shadow:var(--shadow);">
        <i class="fa-solid fa-headset" style="color:white;font-size:1rem;"></i>
      </div>
      <div style="flex:1;">
        <h2 style="font-size:1.125rem;font-weight:700;margin:0;">Support Center</h2>
        <p style="font-size:0.75rem;color:var(--text-muted);margin:0;">3 open conversations</p>
      </div>
    </div>
    <input type="text" class="input" placeholder="Search conversations..." style="width:100%;" />
  </div>"##;

    let conv = |subj: &str, msg: &str, time: &str, unread: u32, status: &str, active: bool| {
        let bg = if active { "background:rgba(168,85,247,0.08);border-left:3px solid var(--epsx-purple);" } else { "" };
        let bold = if unread > 0 { "font-weight:700;" } else { "font-weight:500;" };
        format!(
            r##"<a href="/chat/1" style="text-decoration:none;color:inherit;display:block;padding:1rem 1.5rem;border-bottom:1px solid var(--border);transition:background 0.15s;{bg}">
        <div style="display:flex;justify-content:space-between;align-items:flex-start;gap:0.5rem;margin-bottom:0.25rem;">
          <span style="{bold}font-size:0.9375rem;flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{subj}</span>
          <span style="font-size:0.6875rem;color:var(--text-muted);flex-shrink:0;">{time}</span>
        </div>
        <p style="font-size:0.8125rem;color:var(--text-muted);margin:0 0 0.5rem;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{msg}</p>
        <div style="display:flex;justify-content:space-between;align-items:center;gap:0.5rem;">
          <span class="badge" style="font-size:0.6875rem;background:rgba(6,182,212,0.15);color:var(--epsx-cyan);border:1px solid rgba(6,182,212,0.3);">{status}</span>
          {unread_badge}
        </div>
      </a>"##,
            subj = subj, msg = msg, time = time, bold = bold, bg = bg, status = status,
            unread_badge = if unread > 0 { format!(r##"<span class="badge badge-primary" style="font-size:0.6875rem;border-radius:9999px;">{n}</span>"##, n = unread) } else { String::new() }
        )
    };

    let conversations = [
        conv("Cannot connect wallet", "Try switching to BSC mainnet in your wallet...", "2m", 2, "Open", true),
        conv("Subscription renewal", "Your Pro plan renews automatically on...", "1h", 0, "In Progress", false),
        conv("API rate limit question", "Thanks for the question! The default rate...", "3h", 0, "Open", false),
        conv("Feature request: dark mode", "Great suggestion! We've added it to our...", "1d", 0, "Resolved", false),
    ].join("");

    let new_btn = r##"<div style="padding:1rem;border-top:1px solid var(--border);">
    <button class="btn btn-gradient btn-block"><i class="fa-solid fa-plus"></i> New Conversation</button>
  </div>"##;

    let header = r##"<div style="padding:1.5rem 2rem;background:var(--gradient-brand);color:white;">
    <div style="display:flex;align-items:center;gap:0.75rem;">
      <button class="nav-link" style="color:white;"><i class="fa-solid fa-arrow-left"></i></button>
      <div style="flex:1;">
        <h2 style="font-size:1.125rem;font-weight:700;margin:0;">Cannot connect wallet</h2>
        <div style="display:flex;align-items:center;gap:0.5rem;margin-top:0.25rem;">
          <span style="font-size:0.75rem;background:rgba(255,255,255,0.2);padding:0.125rem 0.5rem;border-radius:9999px;">Open</span>
          <span style="font-size:0.75rem;opacity:0.8;">Technical Support</span>
        </div>
      </div>
      <button class="btn btn-sm" style="background:rgba(255,255,255,0.2);color:white;border:1px solid rgba(255,255,255,0.3);"><i class="fa-solid fa-check"></i> Resolve</button>
    </div>
  </div>"##;

    let message = |mine: bool, content: &str, time: &str| {
        let align = if mine { "align-items:flex-end;" } else { "align-items:flex-start;" };
        let bg = if mine { "background:var(--gradient-brand);color:white;" } else { "background:var(--bg-secondary);color:var(--text);" };
        let max = "max-width:70%;";
        format!(
            r##"<div style="display:flex;flex-direction:column;{align}margin-bottom:1rem;">
        <div style="{max}padding:0.75rem 1rem;border-radius:1rem;{bg}font-size:0.9375rem;line-height:1.5;">{content}</div>
        <span style="font-size:0.6875rem;color:var(--text-subtle);margin-top:0.25rem;">{time}</span>
      </div>"##,
            align = align, max = max, bg = bg, content = content, time = time
        )
    };

    let messages = format!(
        r##"<div style="flex:1;padding:1.5rem 2rem;overflow-y:auto;display:flex;flex-direction:column;">
        {a}
        {b}
        {c}
      </div>
      <div style="padding:1rem 2rem;border-top:1px solid var(--border);display:flex;gap:0.75rem;align-items:flex-end;">
        <textarea class="input" rows="2" placeholder="Type a message..." style="flex:1;resize:none;"></textarea>
        <button class="btn btn-gradient"><i class="fa-solid fa-paper-plane"></i></button>
      </div>"##,
        a = message(false, "Hi! I'm having trouble connecting my MetaMask wallet to EPSX. It keeps saying wrong chain.", "10:23 AM"),
        b = message(true, "Hello! Make sure your wallet is set to BSC mainnet (chain ID 56). You can switch networks in MetaMask's network selector.", "10:25 AM"),
        c = message(false, "Got it, switching now... yes! That worked. Thank you!", "10:27 AM"),
    );

    format!(
        r##"<section style="height:calc(100vh - 4rem);display:flex;">
<div style="width:20rem;flex-shrink:0;border-right:1px solid var(--border);display:flex;flex-direction:column;background:var(--bg-secondary);">
  {sidebar_header}
  <div style="flex:1;overflow-y:auto;">{conversations}</div>
  {new_btn}
</div>
<div style="flex:1;display:flex;flex-direction:column;background:var(--bg);min-width:0;">
  {header}
  {messages}
</div>
</section>"##,
        sidebar_header = sidebar_header,
        conversations = conversations,
        new_btn = new_btn,
        header = header,
        messages = messages,
    )
}

// =====================================================================
// /developer + /developer/docs + /developer/usage
// =====================================================================

pub fn developer_overview_body() -> String {
    let stat1 = StatCard::new("API Access", "Active").icon("key", "var(--epsx-green)").href("/developer/usage").render();
    let stat2 = StatCard::new("Rate Limit", "60/min · 10K/day").icon("gauge-high", "var(--epsx-blue-start)").render();
    let stat3 = StatCard::new("Total Usage", "1,247").icon("chart-bar", "var(--epsx-purple)").href("/developer/usage").render();
    let stat4 = StatCard::new("Expires", "June 30, 2026").change("21 days left", BadgeKind::Info).icon("calendar", "var(--epsx-amber)").render();

    let key_card = r##"<div class="card-insight" style="padding:1.5rem;">
    <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:1rem;">
      <div>
        <h3 style="font-size:1.125rem;font-weight:700;margin:0 0 0.25rem;">Production API Key</h3>
        <code style="font-size:0.75rem;color:var(--text-muted);font-family:monospace;">epsx_live_a1b2...c3d4</code>
      </div>
      <span class="badge badge-success">Active</span>
    </div>
    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;margin-bottom:1rem;">
      <span class="badge badge-info" style="font-size:0.6875rem;">analytics:read</span>
      <span class="badge badge-info" style="font-size:0.6875rem;">portfolio:read</span>
      <span class="badge badge-info" style="font-size:0.6875rem;">wallet:read</span>
    </div>
    <div style="display:flex;gap:0.5rem;">
      <button class="btn btn-outline btn-sm" style="flex:1;"><i class="fa-solid fa-copy"></i> Copy</button>
      <button class="btn btn-danger btn-sm" style="flex:1;"><i class="fa-solid fa-trash"></i> Revoke</button>
    </div>
  </div>"##;

    let security = r##"<div class="card-insight" style="padding:1.5rem;background:rgba(59,130,246,0.05);border-color:rgba(59,130,246,0.3);">
    <h3 style="font-size:1.125rem;font-weight:700;margin-bottom:0.75rem;display:flex;align-items:center;gap:0.5rem;">
      <i class="fa-solid fa-shield-halved" style="color:var(--epsx-blue-start);"></i> Security Best Practices
    </h3>
    <ul style="list-style:none;padding:0;margin:0;display:grid;gap:0.5rem;font-size:0.875rem;">
      <li style="display:flex;gap:0.5rem;align-items:flex-start;"><i class="fa-solid fa-check" style="color:var(--epsx-green);margin-top:0.25rem;"></i> Never commit API keys to version control</li>
      <li style="display:flex;gap:0.5rem;align-items:flex-start;"><i class="fa-solid fa-check" style="color:var(--epsx-green);margin-top:0.25rem;"></i> Use environment variables for keys</li>
      <li style="display:flex;gap:0.5rem;align-items:flex-start;"><i class="fa-solid fa-check" style="color:var(--epsx-green);margin-top:0.25rem;"></i> Rotate keys every 90 days</li>
      <li style="display:flex;gap:0.5rem;align-items:flex-start;"><i class="fa-solid fa-check" style="color:var(--epsx-green);margin-top:0.25rem;"></i> Use the minimum required permissions</li>
    </ul>
  </div>"##;

    let create_form = r##"<div class="card-insight" style="padding:1.5rem;">
    <h3 style="font-size:1.125rem;font-weight:700;margin-bottom:1rem;"><i class="fa-solid fa-plus"></i> Create New API Key</h3>
    <div style="display:grid;gap:1rem;">
      <div>
        <label class="label">Key Name</label>
        <input type="text" class="input" placeholder="e.g. Production Backend" />
      </div>
      <div>
        <label class="label">Permissions</label>
        <div style="display:grid;gap:0.5rem;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;">
          <label style="display:flex;align-items:center;gap:0.5rem;cursor:pointer;"><input type="checkbox" checked /> <span style="font-size:0.875rem;">analytics:read</span></label>
          <label style="display:flex;align-items:center;gap:0.5rem;cursor:pointer;"><input type="checkbox" /> <span style="font-size:0.875rem;">portfolio:read</span></label>
          <label style="display:flex;align-items:center;gap:0.5rem;cursor:pointer;"><input type="checkbox" /> <span style="font-size:0.875rem;">wallet:read</span></label>
          <label style="display:flex;align-items:center;gap:0.5rem;cursor:pointer;"><input type="checkbox" /> <span style="font-size:0.875rem;">subscription:write</span></label>
        </div>
      </div>
      <button class="btn btn-gradient"><i class="fa-solid fa-key"></i> Generate Key</button>
    </div>
  </div>"##;

    format!(
        r##"<section class="section">
<div class="container-x">
  <div style="margin-bottom:2rem;">
    <span class="badge-pill"><i class="fa-solid fa-code" style="color:var(--epsx-orange);"></i> Developer</span>
    <h1 style="font-size:2.5rem;font-weight:800;margin:1rem 0 0.5rem;">API Dashboard</h1>
    <p style="color:var(--text-muted);">Manage your API keys, monitor usage, and explore the documentation.</p>
  </div>

  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(200px, 1fr));gap:1rem;margin-bottom:2rem;">
    {s1}
    {s2}
    {s3}
    {s4}
  </div>

  <div style="display:grid;grid-template-columns:2fr 1fr;gap:1.5rem;margin-bottom:1.5rem;">
    {key_card}
    {security}
  </div>

  {create_form}
</div>
</section>"##,
        s1 = stat1, s2 = stat2, s3 = stat3, s4 = stat4,
        key_card = key_card, security = security, create_form = create_form
    )
}

pub fn developer_docs_body() -> String {
    let sidebar = r##"<nav style="position:sticky;top:5rem;padding:1.5rem;background:var(--bg-secondary);border-radius:1rem;height:fit-content;">
    <h3 style="font-size:0.75rem;font-weight:700;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;margin-bottom:0.75rem;">Getting Started</h3>
    <ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.25rem;">
      <li><a href="#auth" class="nav-link" style="width:100%;">Authentication</a></li>
    </ul>
    <h3 style="font-size:0.75rem;font-weight:700;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;margin-bottom:0.75rem;">Endpoints</h3>
    <ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.25rem;">
      <li><a href="#analytics" class="nav-link" style="width:100%;">Analytics</a></li>
      <li><a href="#portfolio" class="nav-link" style="width:100%;">Portfolio</a></li>
      <li><a href="#user" class="nav-link" style="width:100%;">User</a></li>
    </ul>
  </nav>"##;

    let auth_section = r##"<section id="auth" style="margin-bottom:3rem;">
    <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:1rem;">
      <span class="badge badge-info">POST</span>
      <code style="font-size:0.875rem;color:var(--text-muted);">/api/v1/auth/siwe</code>
    </div>
    <h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Authentication</h2>
    <p style="color:var(--text-muted);margin-bottom:1rem;">Sign in with your Ethereum wallet using Sign-In with Ethereum (SIWE). Returns a JWT access token.</p>
    <div class="card-insight" style="font-family:monospace;font-size:0.875rem;background:var(--bg-secondary);">
      <div style="color:var(--text-muted);margin-bottom:0.5rem;">// Request</div>
      <pre style="margin:0;color:var(--text);">curl -X POST https://api.epsx.io/api/v1/auth/siwe \
  -H "Content-Type: application/json" \
  -d '{"message":"...","signature":"0x..."}'</pre>
    </div>
  </section>"##;

    let analytics_section = r##"<section id="analytics" style="margin-bottom:3rem;">
    <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:1rem;">
      <span class="badge badge-success">GET</span>
      <code style="font-size:0.875rem;color:var(--text-muted);">/api/v1/analytics/rankings</code>
    </div>
    <h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Get Rankings</h2>
    <p style="color:var(--text-muted);margin-bottom:1rem;">Returns the top-ranked stocks and tokens by EPS growth.</p>
    <h3 style="font-size:1rem;font-weight:600;margin:1rem 0 0.5rem;">Query Parameters</h3>
    <table style="width:100%;border-collapse:collapse;font-size:0.875rem;">
      <thead><tr style="text-align:left;border-bottom:1px solid var(--border);"><th style="padding:0.5rem 0.75rem;">Name</th><th style="padding:0.5rem 0.75rem;">Type</th><th style="padding:0.5rem 0.75rem;">Description</th></tr></thead>
      <tbody>
        <tr style="border-bottom:1px solid var(--border);"><td style="padding:0.625rem 0.75rem;font-family:monospace;">page</td><td style="padding:0.625rem 0.75rem;">integer</td><td style="padding:0.625rem 0.75rem;">Page number (default: 1)</td></tr>
        <tr style="border-bottom:1px solid var(--border);"><td style="padding:0.625rem 0.75rem;font-family:monospace;">limit</td><td style="padding:0.625rem 0.75rem;">integer</td><td style="padding:0.625rem 0.75rem;">Results per page (default: 20, max: 100)</td></tr>
        <tr><td style="padding:0.625rem 0.75rem;font-family:monospace;">country</td><td style="padding:0.625rem 0.75rem;">string</td><td style="padding:0.625rem 0.75rem;">Filter by country code</td></tr>
      </tbody>
    </table>
  </section>"##;

    let portfolio_section = r##"<section id="portfolio" style="margin-bottom:3rem;">
    <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:1rem;">
      <span class="badge badge-success">GET</span>
      <code style="font-size:0.875rem;color:var(--text-muted);">/api/v1/portfolio/watchlist</code>
    </div>
    <h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Get Watchlist</h2>
    <p style="color:var(--text-muted);">Returns the authenticated user's watchlist of tracked symbols.</p>
  </section>"##;

    let user_section = r##"<section id="user" style="margin-bottom:3rem;">
    <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:1rem;">
      <span class="badge badge-success">GET</span>
      <code style="font-size:0.875rem;color:var(--text-muted);">/api/v1/user/profile</code>
    </div>
    <h2 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">Get Profile</h2>
    <p style="color:var(--text-muted);">Returns the authenticated user's profile, including permissions and plan access.</p>
  </section>"##;

    let try_it = r##"<div class="card-insight" style="padding:1.5rem;background:rgba(168,85,247,0.05);border-color:rgba(168,85,247,0.3);">
    <h3 style="font-size:1.125rem;font-weight:700;margin-bottom:1rem;display:flex;align-items:center;gap:0.5rem;">
      <i class="fa-solid fa-play" style="color:var(--epsx-purple);"></i> Try It
    </h3>
    <div style="margin-bottom:1rem;">
      <label class="label">Your API Key</label>
      <select class="input">
        <option>Production API Key (epsx_live_a1b2...c3d4)</option>
      </select>
    </div>
    <button class="btn btn-gradient"><i class="fa-solid fa-paper-plane"></i> Send Request</button>
  </div>"##;

    format!(
        r##"<section class="section">
<div class="container-x">
  <div style="margin-bottom:2rem;">
    <div style="width:4rem;height:0.25rem;background:var(--gradient-warm);border-radius:9999px;margin-bottom:1rem;"></div>
    <h1 class="gradient-text" style="font-size:2.5rem;font-weight:800;margin-bottom:0.5rem;">API Reference</h1>
    <p style="color:var(--text-muted);font-size:1.125rem;">Complete reference for the EPSX public API.</p>
  </div>

  <div style="display:grid;grid-template-columns:14rem 1fr;gap:2rem;">
    {sidebar}
    <div>
      {auth}
      {analytics}
      {portfolio}
      {user}
      {try_it}
    </div>
  </div>
</div>
</section>"##,
        sidebar = sidebar,
        auth = auth_section,
        analytics = analytics_section,
        portfolio = portfolio_section,
        user = user_section,
        try_it = try_it
    )
}

pub fn developer_usage_body() -> String {
    let s1 = StatCard::new("Total Requests", "12,847").change("+12.4% this week", BadgeKind::Success).icon("chart-bar", "var(--epsx-blue-start)").render();
    let s2 = StatCard::new("Requests (24h)", "1,247").icon("clock", "var(--epsx-green)").render();
    let s3 = StatCard::new("Error Rate (24h)", "0.32%").change("-0.1%", BadgeKind::Success).icon("triangle-exclamation", "var(--epsx-amber)").render();
    let s4 = StatCard::new("Success Rate", "99.68%").change("+0.1%", BadgeKind::Success).icon("circle-check", "var(--epsx-green)").render();

    let tabs = Tabs::new("range").tab("7d", "7 days").tab("30d", "30 days").tab("90d", "90 days").active("30d").render();

    let bar = |height: u32, label: &str| {
        format!(r##"<div style="display:flex;flex-direction:column;align-items:center;gap:0.25rem;flex:1;">
        <div style="width:100%;background:var(--gradient-warm);height:{height}px;border-radius:0.25rem 0.25rem 0 0;transition:height 0.3s;"></div>
        <span style="font-size:0.6875rem;color:var(--text-muted);">{label}</span>
      </div>"##, height = height, label = label)
    };

    let bars = [
        bar(40, "Mon"), bar(55, ""), bar(38, ""), bar(72, "Wed"),
        bar(60, ""), bar(48, "Fri"), bar(80, ""),
        bar(50, "Mon"), bar(65, ""), bar(58, "Wed"),
        bar(82, ""), bar(70, "Fri"), bar(95, ""),
        bar(60, "Mon"), bar(75, ""), bar(68, "Wed"),
        bar(90, ""), bar(78, "Fri"), bar(110, ""),
    ].join("");

    format!(
        r##"<section class="section">
<div class="container-x">
  <div style="margin-bottom:2rem;">
    <span class="badge-pill"><i class="fa-solid fa-chart-line" style="color:var(--epsx-orange);"></i> Usage</span>
    <h1 style="font-size:2.5rem;font-weight:800;margin:1rem 0 0.5rem;">API Usage</h1>
    <p style="color:var(--text-muted);">Monitor your API consumption and rate limits.</p>
  </div>

  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(200px, 1fr));gap:1rem;margin-bottom:2rem;">
    {s1}
    {s2}
    {s3}
    {s4}
  </div>

  <div class="card-insight" style="padding:1.5rem;margin-bottom:1.5rem;">
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1.5rem;">
      <h2 style="font-size:1.25rem;font-weight:700;margin:0;">Usage History</h2>
      {tabs}
    </div>
    <div style="display:flex;align-items:flex-end;gap:0.5rem;height:10rem;padding:1rem 0;border-bottom:1px solid var(--border);">
      {bars}
    </div>
  </div>

  <div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;">Top Endpoints</h2>
    <table style="width:100%;border-collapse:collapse;font-size:0.875rem;">
      <thead><tr style="text-align:left;border-bottom:1px solid var(--border);"><th style="padding:0.625rem 0;">Method</th><th style="padding:0.625rem 0;">Endpoint</th><th style="padding:0.625rem 0;text-align:right;">Count</th></tr></thead>
      <tbody>
        <tr style="border-bottom:1px solid var(--border);"><td style="padding:0.75rem 0;"><span class="badge badge-success" style="font-size:0.6875rem;">GET</span></td><td style="padding:0.75rem 0;font-family:monospace;">/api/v1/analytics/rankings</td><td style="padding:0.75rem 0;text-align:right;font-weight:600;">8,234</td></tr>
        <tr style="border-bottom:1px solid var(--border);"><td style="padding:0.75rem 0;"><span class="badge badge-success" style="font-size:0.6875rem;">GET</span></td><td style="padding:0.75rem 0;font-family:monospace;">/api/v1/portfolio/watchlist</td><td style="padding:0.75rem 0;text-align:right;font-weight:600;">2,145</td></tr>
        <tr><td style="padding:0.75rem 0;"><span class="badge badge-info" style="font-size:0.6875rem;">POST</span></td><td style="padding:0.75rem 0;font-family:monospace;">/api/v1/auth/refresh</td><td style="padding:0.75rem 0;text-align:right;font-weight:600;">1,468</td></tr>
      </tbody>
    </table>
  </div>
</div>
</section>"##,
        s1 = s1, s2 = s2, s3 = s3, s4 = s4, tabs = tabs, bars = bars
    )
}

// =====================================================================
// /account + /account/credits + /profile
// =====================================================================

pub fn account_body() -> String {
    // Matches epsx.io's actual /account page (👤 Account Settings)
    r##"<section class="section">
<div class="container-x" style="max-width:64rem;">
  <div style="text-align:center;margin-bottom:2.5rem;">
    <h1 style="font-size:2.5rem;font-weight:800;margin:0 0 0.75rem;color:var(--text);">👤 Account Settings</h1>
    <p style="color:var(--text-muted);font-size:1rem;margin:0;">Manage your account access, payments, and preferences with ease</p>
  </div>

  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;margin-bottom:2.5rem;">
    <div class="card-glass" style="padding:1.25rem;">
      <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.5rem;">
        <i data-lucide="wallet" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;"></i>
        <span style="font-size:0.75rem;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">Wallet</span>
      </div>
      <div style="font-size:0.875rem;font-weight:600;color:var(--text);">Current Address</div>
      <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.25rem;">Not Connected</div>
    </div>
    <div class="card-glass" style="padding:1.25rem;">
      <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.5rem;">
        <i data-lucide="check-circle" style="color:var(--epsx-green);width:1.25rem;height:1.25rem;"></i>
        <span style="font-size:0.75rem;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">Active</span>
      </div>
      <div style="font-size:0.875rem;font-weight:600;color:var(--text);">Member Since</div>
      <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.25rem;">Join Now</div>
    </div>
    <div class="card-glass" style="padding:1.25rem;">
      <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.5rem;">
        <i data-lucide="coins" style="color:var(--epsx-amber);width:1.25rem;height:1.25rem;"></i>
        <span style="font-size:0.75rem;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">Credits</span>
      </div>
      <div style="font-size:0.875rem;font-weight:600;color:var(--text);">Available Balance</div>
      <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.25rem;">$0</div>
    </div>
    <div class="card-glass" style="padding:1.25rem;">
      <div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.5rem;">
        <i data-lucide="shield" style="color:var(--epsx-green);width:1.25rem;height:1.25rem;"></i>
        <span style="font-size:0.75rem;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">Secure</span>
      </div>
      <div style="font-size:0.875rem;font-weight:600;color:var(--text);">Method</div>
      <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.25rem;">Web3 Vault</div>
    </div>
  </div>

  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;margin-bottom:2.5rem;">
    <a href="/chat" class="card-glass" style="padding:1.25rem;text-decoration:none;color:inherit;display:block;transition:transform 0.2s ease;" onmouseover="this.style.transform='translateY(-2px)'" onmouseout="this.style.transform=''">
      <i data-lucide="life-buoy" style="color:var(--epsx-cyan);width:1.5rem;height:1.5rem;margin-bottom:0.5rem;"></i>
      <div style="font-size:0.875rem;font-weight:600;color:var(--text);">🛟 Support Center</div>
      <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.25rem;">Need help? Connect with our team</div>
      <div style="font-size:0.75rem;color:var(--epsx-cyan);margin-top:0.5rem;font-weight:500;">Contact &rarr;</div>
    </a>
    <a href="/profile" class="card-glass" style="padding:1.25rem;text-decoration:none;color:inherit;display:block;transition:transform 0.2s ease;" onmouseover="this.style.transform='translateY(-2px)'" onmouseout="this.style.transform=''">
      <i data-lucide="lock" style="color:var(--epsx-purple);width:1.5rem;height:1.5rem;margin-bottom:0.5rem;"></i>
      <div style="font-size:0.875rem;font-weight:600;color:var(--text);">🔒 Privacy Control</div>
      <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.25rem;">Manage your data and visibility settings</div>
      <div style="font-size:0.75rem;color:var(--epsx-purple);margin-top:0.5rem;font-weight:500;">Settings &rarr;</div>
    </a>
    <a href="/notifications" class="card-glass" style="padding:1.25rem;text-decoration:none;color:inherit;display:block;transition:transform 0.2s ease;" onmouseover="this.style.transform='translateY(-2px)'" onmouseout="this.style.transform=''">
      <i data-lucide="bell" style="color:var(--epsx-orange);width:1.5rem;height:1.5rem;margin-bottom:0.5rem;"></i>
      <div style="font-size:0.875rem;font-weight:600;color:var(--text);">🔔 Recent Activity</div>
      <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.25rem;">Check your latest logs and alerts</div>
      <div style="font-size:0.75rem;color:var(--epsx-orange);margin-top:0.5rem;font-weight:500;">View Logs &rarr;</div>
    </a>
  </div>

  <h2 style="font-size:1.5rem;font-weight:700;margin:0 0 1rem;color:var(--text);">Access &amp; Plans</h2>
  <div class="card-glass" style="padding:1.5rem;margin-bottom:2rem;">
    <p style="color:var(--text-muted);font-size:0.875rem;margin:0;">Unable to load access details.</p>
  </div>

  <h2 style="font-size:1.5rem;font-weight:700;margin:2rem 0 1rem;color:var(--text);">Transaction History</h2>
  <div class="card-glass" style="padding:1.5rem;margin-bottom:1rem;">
    <p style="color:var(--text-muted);font-size:0.875rem;margin:0;">📜</p>
  </div>

  <h2 style="font-size:1.5rem;font-weight:700;margin:2rem 0 1rem;color:var(--text);">Payment Records</h2>
  <div class="card-glass" style="padding:1.5rem;margin-bottom:1rem;">
    <p style="color:var(--text-muted);font-size:0.875rem;margin:0 0 0.5rem;">0 total transactions</p>
    <div style="display:flex;gap:0.5rem;align-items:center;">
      <p style="color:var(--epsx-red);font-size:0.875rem;margin:0;">History Load Failed</p>
    </div>
    <p style="color:var(--text-muted);font-size:0.75rem;margin:0.5rem 0 1rem;">Unable to load payment history. Please try again later.</p>
    <button class="btn btn-outline btn-sm">Try Again</button>
  </div>

  <h2 style="font-size:1.5rem;font-weight:700;margin:2rem 0 1rem;color:var(--text);">Notification Preferences</h2>
  <div class="card-glass" style="padding:1.5rem;margin-bottom:1rem;">
    <p style="color:var(--text-muted);font-size:0.875rem;margin:0 0 1rem;">Choose exactly what you want to be notified about. We'll send alerts via web push to keep you updated.</p>
    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;margin-bottom:1rem;">
      <a href="/notifications" class="btn btn-outline btn-sm">Browse All Alerts</a>
      <button class="btn btn-outline btn-sm">Advanced Settings</button>
    </div>
    <div style="display:grid;gap:0.75rem;">
      <label style="display:flex;align-items:center;justify-content:space-between;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;cursor:pointer;">
        <span style="display:flex;align-items:center;gap:0.75rem;"><i data-lucide="trending-up" style="color:var(--epsx-cyan);width:1rem;height:1rem;"></i> <span><strong style="font-size:0.875rem;">💹 Analytics Alerts</strong><br><span style="font-size:0.75rem;color:var(--text-muted);">Price movements &amp; portfolio</span></span></span>
        <input type="checkbox" checked />
      </label>
      <label style="display:flex;align-items:center;justify-content:space-between;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;cursor:pointer;">
        <span style="display:flex;align-items:center;gap:0.75rem;"><i data-lucide="shield" style="color:var(--epsx-green);width:1rem;height:1rem;"></i> <span><strong style="font-size:0.875rem;">🛡️ Security Alerts</strong><br><span style="font-size:0.75rem;color:var(--text-muted);">Auth &amp; security warnings</span></span></span>
        <input type="checkbox" checked />
      </label>
      <label style="display:flex;align-items:center;justify-content:space-between;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;cursor:pointer;">
        <span style="display:flex;align-items:center;gap:0.75rem;"><i data-lucide="user" style="color:var(--epsx-orange);width:1rem;height:1rem;"></i> <span><strong style="font-size:0.875rem;">👤 Account Updates</strong><br><span style="font-size:0.75rem;color:var(--text-muted);">Profile &amp; subscription</span></span></span>
        <input type="checkbox" checked />
      </label>
      <label style="display:flex;align-items:center;justify-content:space-between;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;cursor:pointer;">
        <span style="display:flex;align-items:center;gap:0.75rem;"><i data-lucide="settings" style="color:var(--text-muted);width:1rem;height:1rem;"></i> <span><strong style="font-size:0.875rem;">⚙️ System Status</strong><br><span style="font-size:0.75rem;color:var(--text-muted);">Maintenance &amp; features</span></span></span>
        <input type="checkbox" />
      </label>
      <label style="display:flex;align-items:center;justify-content:space-between;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;cursor:pointer;">
        <span style="display:flex;align-items:center;gap:0.75rem;"><i data-lucide="gift" style="color:var(--epsx-pink);width:1rem;height:1rem;"></i> <span><strong style="font-size:0.875rem;">🎁 Promotions</strong><br><span style="font-size:0.75rem;color:var(--text-muted);">News &amp; special offers</span></span></span>
        <input type="checkbox" />
      </label>
    </div>
  </div>

  <div class="card-glass" style="padding:1.5rem;margin-top:2rem;background:linear-gradient(135deg, rgba(59,130,246,0.05), rgba(168,85,247,0.05));">
    <h3 style="font-size:1.125rem;font-weight:700;margin:0 0 0.5rem;color:var(--text);">🔒 Privacy &amp; Data Security</h3>
    <p style="color:var(--text-muted);font-size:0.875rem;line-height:1.6;margin:0 0 0.75rem;">Your account data is secured with industrial-grade encryption and protocol-level security.</p>
    <a href="/privacy" style="color:var(--epsx-cyan);font-size:0.875rem;font-weight:500;">Read Policy &rarr;</a>
  </div>
</div>
</section>"##.to_string()
}

pub fn account_credits_body() -> String {
    let balance = r##"<div class="card-insight" style="padding:2.5rem;text-align:center;background:var(--gradient-warm);color:white;border:none;">
    <div style="font-size:0.875rem;opacity:0.85;margin-bottom:0.5rem;">AVAILABLE BALANCE</div>
    <div style="font-size:4rem;font-weight:900;line-height:1;margin-bottom:0.5rem;">1,250</div>
    <div style="font-size:1.125rem;opacity:0.9;">Credits</div>
  </div>"##;

    let history = r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;">Transaction History</h2>
    <div style="display:grid;gap:0.5rem;">
      <div style="display:flex;justify-content:space-between;align-items:center;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;">
        <div><div style="font-weight:600;">API Call Refund</div><div style="font-size:0.75rem;color:var(--text-muted);">June 8, 2026</div></div>
        <span style="color:var(--epsx-green);font-weight:700;">+25</span>
      </div>
      <div style="display:flex;justify-content:space-between;align-items:center;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;">
        <div><div style="font-weight:600;">Pro Plan Bonus</div><div style="font-size:0.75rem;color:var(--text-muted);">June 1, 2026</div></div>
        <span style="color:var(--epsx-green);font-weight:700;">+500</span>
      </div>
      <div style="display:flex;justify-content:space-between;align-items:center;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;">
        <div><div style="font-weight:600;">Premium Support Used</div><div style="font-size:0.75rem;color:var(--text-muted);">May 28, 2026</div></div>
        <span style="color:var(--epsx-red);font-weight:700;">-10</span>
      </div>
    </div>
  </div>"##;

    format!(
        r##"<section class="section">
<div class="container-x" style="max-width:48rem;">
  <div style="text-align:center;margin-bottom:2rem;">
    <span class="badge-pill"><i class="fa-solid fa-coins" style="color:var(--epsx-orange);"></i> Credits</span>
    <h1 style="font-size:2.5rem;font-weight:800;margin:1rem 0 0.5rem;">Credit Balance</h1>
    <p style="color:var(--text-muted);">Spend credits on premium features and API calls.</p>
  </div>
  {balance}
  <div style="margin-top:2rem;">{history}</div>
</div>
</section>"##,
        balance = balance, history = history
    )
}

pub fn profile_body() -> String {
    let user_card = r##"<aside class="card-insight" style="padding:2rem;text-align:center;position:sticky;top:5rem;">
    <div style="width:6rem;height:6rem;border-radius:9999px;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;box-shadow:var(--shadow-orange);font-size:2rem;color:white;font-weight:800;">JD</div>
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 0.25rem;">Jane Doe</h2>
    <p style="color:var(--text-muted);font-size:0.875rem;margin:0 0 0.5rem;">jane@example.com</p>
    <span class="badge badge-success" style="margin-bottom:1rem;">Verified</span>
    <div style="display:grid;gap:0.5rem;text-align:left;padding-top:1rem;border-top:1px solid var(--border);">
      <div style="display:flex;justify-content:space-between;font-size:0.875rem;"><span style="color:var(--text-muted);">Role</span><span style="font-weight:600;">Pro User</span></div>
      <div style="display:flex;justify-content:space-between;font-size:0.875rem;"><span style="color:var(--text-muted);">Permissions</span><span style="font-weight:600;">12</span></div>
      <div style="display:flex;justify-content:space-between;font-size:0.875rem;"><span style="color:var(--text-muted);">Platform</span><span style="font-weight:600;">EPSX</span></div>
    </div>
    <a href="/plans" class="btn btn-gradient btn-block" style="margin-top:1.5rem;"><i class="fa-solid fa-arrow-up"></i> Upgrade Access</a>
  </aside>"##;

    let tabs = Tabs::new("profile").tab("web3", "Web3").tab("account", "Account").tab("email", "Email").tab("data", "Data").active("web3").render();

    let web3 = r##"<div class="card-insight" style="padding:1.5rem;">
    <h3 style="font-size:1.125rem;font-weight:700;margin:0 0 1rem;"><i class="fa-solid fa-wallet"></i> Wallet Information</h3>
    <div style="display:grid;gap:1rem;">
      <div><label class="label">Wallet ID</label>
        <code style="display:block;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;font-size:0.875rem;">0x1234567890abcdef...c3d4</code>
      </div>
      <div><label class="label">Email</label>
        <code style="display:block;padding:0.75rem;background:var(--bg-secondary);border-radius:0.5rem;font-size:0.875rem;">jane@example.com</code>
      </div>
      <div><label class="label">Access Group</label>
        <span class="badge badge-primary">Pro Plan</span>
      </div>
    </div>
  </div>"##;

    let account = r##"<div class="card-insight" style="padding:1.5rem;">
    <h3 style="font-size:1.125rem;font-weight:700;margin:0 0 1rem;"><i class="fa-solid fa-user-gear"></i> Account Settings</h3>
    <p style="color:var(--text-muted);margin-bottom:1rem;">Update your display name and preferences.</p>
    <div style="display:grid;gap:1rem;">
      {name_input}
      {bio_input}
      <button class="btn btn-gradient"><i class="fa-solid fa-save"></i> Save Changes</button>
    </div>
  </div>"##;

    let name_input = Input::new("display_name").label("Display Name").placeholder("Your name").value("Jane Doe").render();
    let bio_input = Input::new("bio").label("Bio").textarea().rows(3).placeholder("Tell us about yourself").value("").render();

    format!(
        r##"<section class="section">
<div class="container-x">
  <div style="margin-bottom:2rem;">
    <h1 style="font-size:2.5rem;font-weight:800;margin:0 0 0.5rem;">Profile &amp; Settings</h1>
    <p style="color:var(--text-muted);">Manage your account, wallet, and preferences.</p>
  </div>

  <div style="display:grid;grid-template-columns:1fr 3fr;gap:2rem;">
    {user_card}
    <div>
      {tabs}
      <div style="margin-top:1.5rem;">
        {web3}
        {account_replaced}
      </div>
    </div>
  </div>
</div>
</section>"##,
        user_card = user_card,
        tabs = tabs,
        web3 = web3,
        account_replaced = account.replace("{name_input}", &name_input).replace("{bio_input}", &bio_input)
    )
}

// =====================================================================
// /notifications
// =====================================================================

pub fn notifications_body_server(_user_id: &str) -> String {
    r##"<section class="section">
<div class="container-x" style="max-width:56rem;">
  <div style="display:flex;justify-content:space-between;align-items:flex-end;flex-wrap:wrap;gap:1rem;margin-bottom:2rem;">
    <div>
      <span class="badge-pill"><i class="fa-solid fa-bell" style="color:var(--epsx-orange);"></i> Notifications</span>
      <h1 style="font-size:2.5rem;font-weight:800;margin:1rem 0 0.5rem;">Notifications</h1>
      <p id="notif-total" style="color:var(--text-muted);">Loading...</p>
    </div>
    <div style="display:flex;gap:0.5rem;">
      <button id="notif-mark-all" class="btn btn-outline btn-sm"><i class="fa-solid fa-check-double"></i> Mark All Read</button>
      <button id="notif-clear-all" class="btn btn-danger btn-sm"><i class="fa-solid fa-trash"></i> Clear All</button>
    </div>
  </div>
  <div class="card-insight" style="padding:1rem;margin-bottom:1.5rem;display:flex;gap:0.75rem;align-items:center;flex-wrap:wrap;">
    <div style="display:flex;gap:0.5rem;">
      <select id="notif-filter-status" class="input" style="width:auto;">
        <option value="all">All</option>
        <option value="unread">Unread</option>
        <option value="read">Read</option>
      </select>
    </div>
    <select id="notif-filter-type" class="input" style="width:auto;min-width:140px;">
      <option value="all">All types</option>
      <option value="security">Security</option>
      <option value="payment">Payment</option>
      <option value="analytics">Analytics</option>
      <option value="permission">Permission</option>
      <option value="system">System</option>
    </select>
    <select id="notif-filter-priority" class="input" style="width:auto;min-width:140px;">
      <option value="all">All priorities</option>
      <option value="urgent">Urgent</option>
      <option value="high">High</option>
      <option value="normal">Normal</option>
      <option value="low">Low</option>
    </select>
  </div>
  <div id="notif-list" style="display:grid;gap:0.75rem;"></div>
  <div id="notif-empty" style="text-align:center;padding:4rem 2rem;color:var(--text-muted);display:none;">
    <i class="fa-solid fa-bell-slash" style="font-size:2.5rem;margin-bottom:1rem;display:block;color:var(--text-subtle);"></i>
    <p style="font-size:1.125rem;margin-bottom:0.5rem;">No notifications</p>
    <p style="font-size:0.875rem;">You're all caught up!</p>
  </div>
  <div style="display:flex;justify-content:center;align-items:center;gap:1rem;margin-top:2rem;">
    <span style="color:var(--text-muted);font-size:0.875rem;">Unread: <span id="notif-unread">0</span></span>
  </div>
</div>
</section>"##.to_string()
}

// =====================================================================
// /access-denied + /offline + /payment
// =====================================================================

pub fn access_denied_body(reason: &str, required: &str) -> String {
    let required_list = if !required.is_empty() {
        format!(r##"<div class="card-insight" style="padding:1.5rem;text-align:left;margin-top:2rem;">
        <h3 style="font-size:0.875rem;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;margin:0 0 0.75rem;">Required Permissions</h3>
        <ul style="list-style:none;padding:0;margin:0;display:grid;gap:0.5rem;">
          {perms}
        </ul>
      </div>"##, perms = required.split(',').map(|p| format!(r##"<li style="display:flex;align-items:center;gap:0.5rem;font-family:monospace;font-size:0.875rem;"><i class="fa-solid fa-key" style="color:var(--epsx-orange);"></i>{}</li>"##, p.trim())).collect::<Vec<_>>().join(""))
    } else {
        String::new()
    };

    format!(
        r##"<section class="section" style="display:flex;align-items:center;justify-content:center;min-height:80vh;">
<div style="text-align:center;max-width:32rem;">
  <div style="width:5rem;height:5rem;border-radius:9999px;background:rgba(239,68,68,0.1);display:flex;align-items:center;justify-content:center;margin:0 auto 1.5rem;">
    <i class="fa-solid fa-shield-halved" style="font-size:2rem;color:var(--epsx-red);"></i>
  </div>
  <h1 style="font-size:2.5rem;font-weight:800;margin-bottom:1rem;">Access Denied</h1>
  <p style="color:var(--text-muted);font-size:1.125rem;margin-bottom:2rem;">{}</p>
  {required_list}
  <div style="display:flex;gap:1rem;justify-content:center;margin-top:2rem;">
    <a href="/" class="btn btn-outline"><i class="fa-solid fa-house"></i> Go Home</a>
    <a href="/contact" class="btn btn-gradient"><i class="fa-solid fa-envelope"></i> Request Access</a>
  </div>
</div>
</section>"##,
        reason
    )
}

pub fn offline_body() -> String {
    let features = [
        ("check", "var(--epsx-green)", "View cached notifications"),
        ("check", "var(--epsx-green)", "Browse previously loaded analytics"),
        ("check", "var(--epsx-green)", "Access user settings"),
        ("exclamation", "var(--epsx-amber)", "Limited: Real-time data and trading"),
    ].iter().map(|(icon, color, text)| format!(r##"<li style="display:flex;align-items:center;gap:0.75rem;padding:0.5rem 0;"><i class="fa-solid fa-circle-{icon}" style="color:{color};"></i> {text}</li>"##, icon = icon, color = color, text = text)).collect::<Vec<_>>().join("");

    format!(
        r##"<section class="section" style="display:flex;align-items:center;justify-content:center;min-height:80vh;">
<div style="text-align:center;max-width:32rem;">
  <div style="width:5rem;height:5rem;border-radius:9999px;background:rgba(249,115,22,0.1);display:flex;align-items:center;justify-content:center;margin:0 auto 1.5rem;">
    <i class="fa-solid fa-wifi" style="font-size:2rem;color:var(--epsx-orange);transform:rotate(45deg);"></i>
  </div>
  <h1 style="font-size:2.5rem;font-weight:800;margin-bottom:0.5rem;">You're Offline</h1>
  <p style="color:var(--text-muted);font-size:1.125rem;margin-bottom:2rem;">No internet connection detected. Some features are unavailable.</p>
  <div class="card-insight" style="padding:1.5rem;text-align:left;margin-bottom:2rem;">
    <h3 style="font-size:0.875rem;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;margin:0 0 0.75rem;">Available Offline</h3>
    <ul style="list-style:none;padding:0;margin:0;">{features}</ul>
  </div>
  <div style="display:flex;gap:0.75rem;justify-content:center;flex-wrap:wrap;">
    <button class="btn btn-gradient" onclick="location.reload()"><i class="fa-solid fa-rotate-right"></i> Try Again</button>
    <a href="/" class="btn btn-outline"><i class="fa-solid fa-house"></i> Home</a>
    <a href="/notifications" class="btn btn-outline"><i class="fa-solid fa-bell"></i> Notifications</a>
  </div>
</div>
</section>"##,
        features = features
    )
}

pub fn payment_body(plan_id: Option<&str>) -> String {
    let plan_param = plan_id.unwrap_or("");
    let plan_title = if !plan_param.is_empty() { plan_param } else { "Pro" };
    let plan_price = "29";

    let plan_card = format!(r##"<div class="card-insight" style="padding:2rem;border:2px solid var(--epsx-orange);">
    <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:1rem;">
      <h2 style="font-size:1.5rem;font-weight:700;margin:0;">{}</h2>
      <span class="badge badge-primary">Selected</span>
    </div>
    <div class="gradient-text" style="font-size:3rem;font-weight:800;margin-bottom:0.5rem;">${}<span style="font-size:1rem;font-weight:500;color:var(--text-muted);-webkit-text-fill-color:var(--text-muted);">/month</span></div>
    <ul style="list-style:none;padding:0;margin:1.5rem 0;display:grid;gap:0.5rem;font-size:0.875rem;">
      <li style="display:flex;gap:0.5rem;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> 10,000 API calls/day</li>
      <li style="display:flex;gap:0.5rem;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Advanced analytics</li>
      <li style="display:flex;gap:0.5rem;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Priority support</li>
      <li style="display:flex;gap:0.5rem;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i> Custom webhooks</li>
    </ul>
  </div>"##, plan_title, plan_price);

    let token_selector = r##"<div style="margin-bottom:1.5rem;">
    <label class="label">Pay with</label>
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.5rem;">
      <button class="card-insight" style="padding:1rem;cursor:pointer;text-align:center;border:2px solid var(--epsx-orange);">
        <div style="font-size:1.5rem;font-weight:700;margin-bottom:0.25rem;">USDT</div>
        <div style="font-size:0.75rem;color:var(--text-muted);">Balance: 250.00</div>
      </button>
      <button class="card-insight" style="padding:1rem;cursor:pointer;text-align:center;">
        <div style="font-size:1.5rem;font-weight:700;margin-bottom:0.25rem;">USDC</div>
        <div style="font-size:0.75rem;color:var(--text-muted);">Balance: 0.00</div>
      </button>
    </div>
  </div>"##;

    format!(
        r##"<section class="section">
<div class="container-x" style="max-width:48rem;">
  <div style="text-align:center;margin-bottom:2rem;">
    <div style="width:4rem;height:4rem;border-radius:9999px;background:linear-gradient(135deg, #a855f7, #6366f1, #3b82f6);display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;box-shadow:0 20px 25px -5px rgba(168,85,247,0.25);">
      <i class="fa-solid fa-gem" style="color:white;font-size:1.5rem;"></i>
    </div>
    <h1 class="gradient-text" style="font-size:2.5rem;font-weight:800;margin-bottom:0.5rem;">Complete Your Subscription</h1>
    <p style="color:var(--text-muted);font-size:1.125rem;">Pay with stablecoins on BSC. Activates instantly.</p>
  </div>

  {plan_card}
  {token_selector}

  <div class="card-insight" style="padding:1.5rem;display:flex;justify-content:space-between;align-items:center;">
    <div>
      <div style="font-size:0.875rem;color:var(--text-muted);">Total to pay</div>
      <div class="gradient-text" style="font-size:2rem;font-weight:800;">$29.00 USDT</div>
    </div>
    <button class="btn btn-gradient btn-lg"><i class="fa-solid fa-bolt"></i> Pay Now</button>
  </div>

  <div style="display:flex;justify-content:center;gap:1.5rem;margin-top:2rem;flex-wrap:wrap;color:var(--text-muted);font-size:0.8125rem;">
    <span><i class="fa-solid fa-shield-halved" style="color:var(--epsx-green);"></i> Blockchain Secured</span>
    <span><i class="fa-solid fa-bolt" style="color:var(--epsx-amber);"></i> Instant Activation</span>
    <span><i class="fa-solid fa-coins" style="color:var(--epsx-blue-start);"></i> USDT &middot; USDC</span>
  </div>
</div>
</section>"##,
        plan_card = plan_card,
        token_selector = token_selector
    )
}
