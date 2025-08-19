/** @type {import('next-sitemap').IConfig} */
module.exports = {
  siteUrl: process.env.SITE_URL || 'https://admin.epsx.com',
  generateRobotsTxt: true,
  sitemapSize: 5000,
  changefreq: 'daily',
  priority: 0.7,
  exclude: [
    '/api/*',
    '/unauthorized',
    '/access-denied',
    '/_*'
  ],
  robotsTxtOptions: {
    policies: [
      {
        userAgent: '*',
        disallow: [
          '/api/',
          '/unauthorized',
          '/access-denied',
          '/_next/',
        ],
      },
    ],
  },
}