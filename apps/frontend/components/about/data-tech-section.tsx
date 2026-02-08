'use client';

import React from 'react';

import { Card, CardContent, CardHeader } from '@/components/ui';

const DataTechSection: React.FC = () => {
  const features = [
    {
      id: 'collection',
      title: 'Data Collection',
      description:
        'Extract data from sensors, websites, IoT devices, applications, and databases',
      details:
        'This initial data gathering is crucial as the raw data will be used for in-depth analysis later.',
    },
    {
      id: 'storage',
      title: 'Data Storage',
      description:
        'Secure and scalable storage using Cloud Storage and Big Data Repositories',
      details:
        'Handle large volumes of data that can be quickly accessed when needed.',
    },
    {
      id: 'management',
      title: 'Data Management',
      description: 'Organize, verify, and maintain data consistency',
      details:
        'Including data quality management, data cleansing, and integration of data from multiple sources.',
    },
    {
      id: 'processing',
      title: 'Data Processing',
      description: 'Advanced processing with ML and AI for predictive analysis',
      details:
        'Analyze and understand data, predict behaviors or trends from historical data.',
    },
    {
      id: 'analytics',
      title: 'Data Analytics',
      description:
        'In-depth analysis using Predictive, Descriptive, and Prescriptive techniques',
      details:
        'Provide insights valuable for business decisions through various analytical methods.',
    },
    {
      id: 'visualization',
      title: 'Data Visualization',
      description: 'Create interactive dashboards and visual representations',
      details:
        'Help users better understand data insights through visual representations.',
    },
  ];

  return (
    <div className="relative mx-auto my-8 flex w-full max-w-7xl flex-col gap-8 overflow-hidden p-4 sm:my-16 sm:p-6 lg:p-8">
      {/* Floating background decorations */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute top-10 left-10 h-20 w-20 rounded-full bg-gradient-to-br from-orange-400/10 to-yellow-400/10" />
        <div className="absolute right-20 bottom-20 h-16 w-16 rounded-full bg-gradient-to-br from-blue-400/10 to-cyan-400/10" />
        <div className="absolute top-1/2 left-1/4 h-12 w-12 rounded-full bg-gradient-to-br from-purple-400/10 to-pink-400/10" />
      </div>

      {/* Overview Section with improved readability */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        {/* Main Definition Card */}
        <Card className="relative mx-2 overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 backdrop-blur-xl sm:mx-0 lg:col-span-2 dark:border-orange-400/20 dark:bg-slate-800/80">
          <div className="absolute inset-0 bg-gradient-to-br from-orange-50/30 via-transparent to-yellow-50/30 dark:from-orange-900/10 dark:via-transparent dark:to-yellow-900/10" />
          <div className="absolute top-0 right-0 h-32 w-32 rounded-full bg-gradient-to-br from-orange-400/10 to-yellow-400/10 blur-xl" />

          <CardHeader className="relative z-10">
            <h3 className="mb-4 bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-2xl font-bold text-transparent">
              🚀 What is a DataTech Platform?
            </h3>
            <div className="space-y-4 text-gray-600 dark:text-gray-300">
              <p className="text-lg leading-relaxed">
                A{' '}
                <strong className="bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text font-bold text-transparent">
                  DataTech Platform
                </strong>{' '}
                is a comprehensive technology ecosystem designed to handle your
                complete data journey.
              </p>
              <p className="leading-relaxed">
                From initial{' '}
                <span className="font-semibold text-orange-600 dark:text-orange-400">
                  collection and storage
                </span>{' '}
                to advanced{' '}
                <span className="font-semibold text-blue-600 dark:text-blue-400">
                  analysis and visualization
                </span>
                , these platforms integrate cutting-edge tools to maximize data
                value.
              </p>
            </div>
          </CardHeader>
        </Card>

        {/* Key Features Card */}
        <Card className="relative mx-2 overflow-hidden rounded-3xl border border-blue-200/50 bg-white/80 backdrop-blur-xl sm:mx-0 dark:border-blue-400/20 dark:bg-slate-800/80">
          <div className="absolute inset-0 bg-gradient-to-br from-blue-50/30 via-transparent to-cyan-50/30 dark:from-blue-900/10 dark:via-transparent dark:to-cyan-900/10" />
          <div className="absolute bottom-0 left-0 h-24 w-24 rounded-full bg-gradient-to-br from-blue-400/10 to-cyan-400/10 blur-xl" />

          <CardHeader className="relative z-10">
            <h3 className="mb-4 bg-gradient-to-r from-blue-500 to-cyan-500 bg-clip-text text-xl font-bold text-transparent">
              💡 Why It Matters
            </h3>
            <div className="space-y-3 text-sm text-gray-600 dark:text-gray-300">
              <div className="flex items-center gap-2">
                <span className="text-green-500">✓</span>
                <span>Complete data lifecycle management</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-green-500">✓</span>
                <span>Integrated tools & technologies</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-green-500">✓</span>
                <span>Business decision support</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-green-500">✓</span>
                <span>Multi-sector applications</span>
              </div>
            </div>
          </CardHeader>
        </Card>
      </div>

      {/* Features Grid with enhanced PancakeSwap styling */}
      <div className="grid grid-cols-1 gap-6 px-2 sm:grid-cols-2 sm:gap-8 sm:px-0 lg:grid-cols-3">
        {features.map((feature, index) => {
          const gradients = [
            'from-orange-500 to-yellow-500',
            'from-blue-500 to-cyan-500',
            'from-purple-500 to-pink-500',
            'from-green-500 to-emerald-500',
            'from-red-500 to-orange-500',
            'from-indigo-500 to-purple-500',
          ];
          const bgGradients = [
            'from-orange-400/10 to-yellow-400/10',
            'from-blue-400/10 to-cyan-400/10',
            'from-purple-400/10 to-pink-400/10',
            'from-green-400/10 to-emerald-400/10',
            'from-red-400/10 to-orange-400/10',
            'from-indigo-400/10 to-purple-400/10',
          ];

          return (
            <Card
              key={feature.id}
              className="group relative overflow-hidden rounded-2xl border border-orange-200/30 bg-white/80 backdrop-blur-xl dark:border-orange-400/20 dark:bg-slate-800/80"
            >
              {/* Card background decoration */}
              <div
                className={`absolute inset-0 bg-gradient-to-br ${bgGradients[index]} opacity-50`}
              />
              <div
                className={`absolute top-0 right-0 h-20 w-20 bg-gradient-to-br ${bgGradients[index]} rounded-full blur-xl`}
              />

              <CardHeader className="relative z-10">
                <h3
                  className={`bg-gradient-to-r text-xl font-bold sm:text-2xl ${gradients[index]} bg-clip-text text-transparent`}
                >
                  {feature.title}
                </h3>
              </CardHeader>
              <CardContent className="relative z-10">
                <p className="mb-4 leading-relaxed text-gray-600 dark:text-gray-300">
                  {feature.description}
                </p>
                <p className="text-sm leading-relaxed text-gray-500 dark:text-gray-400">
                  {feature.details}
                </p>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Benefits Section with PancakeSwap styling */}
      <Card className="group relative mx-2 mt-8 overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 backdrop-blur-xl sm:mx-0 dark:border-orange-400/20 dark:bg-slate-800/80">
        {/* Card background decoration */}
        <div className="absolute inset-0 bg-gradient-to-br from-green-50/30 via-transparent to-emerald-50/30 dark:from-green-900/10 dark:via-transparent dark:to-emerald-900/10" />
        <div className="absolute bottom-0 left-0 h-40 w-40 rounded-full bg-gradient-to-br from-green-400/10 to-emerald-400/10 blur-2xl" />

        <CardHeader className="relative z-10">
          <h3 className="bg-gradient-to-r from-green-500 to-emerald-500 bg-clip-text text-2xl font-bold text-transparent sm:text-3xl">
            🎯 Benefits
          </h3>
        </CardHeader>
        <CardContent className="relative z-10">
          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
            <div className="space-y-4">
              <p className="flex items-center font-medium text-gray-600 dark:text-gray-300">
                <span className="mr-3 text-xl text-green-500">✅</span>
                Enable accurate and efficient data-driven decisions
              </p>
              <p className="flex items-center font-medium text-gray-600 dark:text-gray-300">
                <span className="mr-3 text-xl text-green-500">⚡</span>
                Increase speed in accessing and processing big data
              </p>
              <p className="flex items-center font-medium text-gray-600 dark:text-gray-300">
                <span className="mr-3 text-xl text-green-500">🔒</span>
                Improve data management organization and security
              </p>
            </div>
            <div className="space-y-4">
              <p className="flex items-center font-medium text-gray-600 dark:text-gray-300">
                <span className="mr-3 text-xl text-green-500">💰</span>
                Reduce costs through cloud systems and scalable storage
              </p>
              <p className="flex items-center font-medium text-gray-600 dark:text-gray-300">
                <span className="mr-3 text-xl text-green-500">🤝</span>
                Support efficient team collaboration in data analysis
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default DataTechSection;
