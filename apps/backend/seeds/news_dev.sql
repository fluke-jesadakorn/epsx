-- ============================================================
-- EPSX NEWS SEED (dev)
-- Run: psql -h localhost -p 5433 -U epsx_user -d epsx_dev -f apps/backend/seeds/news_dev.sql
-- ============================================================

INSERT INTO news_articles (
  title, slug, summary, content, cover_image_url,
  author_wallet, status, tags, published_at, is_pinned, pinned_at
) VALUES

-- 1. Welcome post (pinned)
(
  'Welcome to EPSX — We''re Live',
  'welcome-to-epsx',
  'After months of building, EPSX is officially open. Here''s a quick look at what we made and why.',
  E'## Hey, we''re live.\n\nWe built EPSX because we were tired of tools that felt like they were made for someone else — either too complicated, locked behind paywalls, or just not that useful in practice.\n\nSo we made our own.\n\nEPSX is a platform for tracking and exploring stocks — rankings, watchlists, analytics — without the noise. No popup ads asking you to upgrade every five seconds. No charts that require a finance degree to read.\n\nWe''re still early. There''s a lot more we want to build. But what''s live today is real, tested, and things we actually use ourselves.\n\nHead to [Analytics](/analytics) to start exploring.\n\nWelcome aboard.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["announcement", "welcome"]'::jsonb,
  NOW() - INTERVAL '52 days',
  true,
  NOW() - INTERVAL '52 days'
),

-- 2. How rankings work
(
  'How Rankings Actually Work on EPSX',
  'how-rankings-work',
  'Rankings aren''t magic — here''s a plain-English explanation of what we''re scoring and why.',
  E'## What is a ranking, exactly?\n\nWe get this question a lot, so here''s a straight answer.\n\nThe rankings on EPSX score stocks based on a combination of activity signals — things like volume patterns, price momentum, and relative movement compared to their sector. No single number tells the whole story, so we combine several.\n\nThe score doesn''t tell you what to buy. It just surfaces what''s moving so you don''t have to scroll through hundreds of tickers manually.\n\n## Why do some stocks disappear from the top?\n\nBecause rankings update regularly. A stock that''s trending today might cool off tomorrow — the list reflects what''s happening now, not last week.\n\n## Can I see more than the top 25?\n\nYes — depending on your plan, you can unlock deeper rankings. Check the [Plans](/plans) page to see what''s available.\n\nIf you have questions about how a specific score is calculated, reach out through [Contact](/contact). We''re happy to explain.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["platform", "rankings"]'::jsonb,
  NOW() - INTERVAL '38 days',
  false,
  NULL
),

-- 3. Plans update
(
  'New Plans: Pick the One That Fits',
  'new-plans-pick-what-fits',
  'We simplified our plans. One day, Starter, Lifetime, Company — here''s how to think about which one is right for you.',
  E'## We cleaned up the plans.\n\nWe had some feedback that the old pricing was confusing. Fair point. So we simplified things.\n\nHere''s the short version:\n\n- **One Day** — Try the platform for 24 hours. Good if you just want to poke around before committing.\n- **Starter** — 30 days of access, 25 rankings, the basics covered. Most individual users start here.\n- **Lifetime** — Pay once, done. Full rankings, API access, no renewal headaches.\n- **Company** — If you''re using this with a team or need higher API limits, this is the one.\n\nThere''s also an **API Personal** plan if you''re building something on top of our data.\n\nAll plans include the core platform — the differences are mostly about how deep you can go and how long the access lasts.\n\nSee the full breakdown on the [Plans page](/plans).',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["plans", "update"]'::jsonb,
  NOW() - INTERVAL '25 days',
  false,
  NULL
),

-- 4. Watchlist feature
(
  'The Watchlist Is Here',
  'watchlist-is-here',
  'You can now save stocks and track them in one place. Here''s how to use it.',
  E'## Save what matters, ignore the rest.\n\nThe Watchlist is now live on EPSX.\n\nHere''s what it does: you bookmark any stock from the rankings, and it shows up in your [Portfolio](/portfolio) page in one place. That''s it. No complicated setup.\n\n## How to add a stock\n\n1. Open the analytics view\n2. Find the stock you want\n3. Click the bookmark icon on the card\n\nIt saves instantly. You''ll see it next time you open Portfolio.\n\n## Is it synced across devices?\n\nYes — your watchlist is tied to your account, so it follows you whether you''re on your laptop or phone.\n\n## Can I remove stocks?\n\nClick the same bookmark icon again and it''s gone. No confirmation dialogs, no extra steps.\n\nLet us know if you run into anything weird — we''re still ironing out edge cases.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["feature", "watchlist"]'::jsonb,
  NOW() - INTERVAL '14 days',
  false,
  NULL
),

-- 5. Roadmap / what's next
(
  'What We''re Building Next',
  'what-were-building-next',
  'A quick look at what''s coming — alerts, mobile improvements, and a few things we can''t talk about yet.',
  E'## Here''s what''s on the board.\n\nWe try to be pretty open about what we''re working on. Here''s the current list:\n\n### Alerts (coming soon)\nYou''ll be able to set a threshold on any stock — when it crosses it, we ping you. Email first, push notifications after that.\n\n### Mobile improvements\nThe mobile layout works, but it''s not where we want it. We''re going through every screen and tightening things up.\n\n### Better filtering on rankings\nRight now you can sort and filter by a few things. We''re expanding that — sector, market cap range, custom time windows.\n\n### A thing we''re not ready to announce yet\nThere''s one bigger feature we''re testing internally. We''ll share it when it''s closer to ready. It''s good.\n\n---\n\nIf there''s something you want that isn''t on this list, tell us through [Contact](/contact). We read every message.',
  NULL,
  '0x0000000000000000000000000000000000000000',
  'published',
  '["roadmap", "update"]'::jsonb,
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
