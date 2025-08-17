"use client";

import React from "react";

import { Card, CardHeader, CardContent, CardDescription } from "@/components/ui";

const DataTechSection: React.FC = () => {
  const features = [
    {
      id: "collection",
      title: "Data Collection",
      description: "Extract data from sensors, websites, IoT devices, applications, and databases",
      details: "This initial data gathering is crucial as the raw data will be used for in-depth analysis later."
    },
    {
      id: "storage",
      title: "Data Storage",
      description: "Secure and scalable storage using Cloud Storage and Big Data Repositories",
      details: "Handle large volumes of data that can be quickly accessed when needed."
    },
    {
      id: "management",
      title: "Data Management",
      description: "Organize, verify, and maintain data consistency",
      details: "Including data quality management, data cleansing, and integration of data from multiple sources."
    },
    {
      id: "processing",
      title: "Data Processing",
      description: "Advanced processing with ML and AI for predictive analysis",
      details: "Analyze and understand data, predict behaviors or trends from historical data."
    },
    {
      id: "analytics",
      title: "Data Analytics",
      description: "In-depth analysis using Predictive, Descriptive, and Prescriptive techniques",
      details: "Provide insights valuable for business decisions through various analytical methods."
    },
    {
      id: "visualization",
      title: "Data Visualization",
      description: "Create interactive dashboards and visual representations",
      details: "Help users better understand data insights through visual representations."
    }
  ];

  return (
    <div className="relative flex flex-col gap-8 w-full p-4 sm:p-6 lg:p-8 max-w-7xl mx-auto my-8 sm:my-16 overflow-hidden">
      {/* Floating background decorations */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-10 left-10 w-20 h-20 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full animate-float" />
        <div className="absolute bottom-20 right-20 w-16 h-16 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 rounded-full animate-bounce-gentle" />
        <div className="absolute top-1/2 left-1/4 w-12 h-12 bg-gradient-to-br from-purple-400/10 to-pink-400/10 rounded-full animate-pulse-gentle" />
      </div>

      {/* Section Header with PancakeSwap styling */}
      <div className="relative text-center space-y-6 mb-8 px-4">
        <h2 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent animate-gradient-x">
          🍰 DataTech Platform
        </h2>
        <div className="w-40 h-1 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 mx-auto rounded-full" />
        
        {/* Decorative elements */}
        <div className="flex justify-center items-center gap-3 mt-4">
          <div className="w-2 h-2 bg-orange-400 rounded-full animate-pulse" />
          <div className="w-3 h-3 bg-yellow-400 rounded-full animate-pulse" style={{ animationDelay: '0.5s' }} />
          <div className="w-2 h-2 bg-orange-400 rounded-full animate-pulse" style={{ animationDelay: '1s' }} />
        </div>
      </div>

      {/* Overview Card with PancakeSwap styling */}
      <Card className="relative overflow-hidden transition-all duration-300 hover:shadow-2xl bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border border-orange-200/50 dark:border-orange-400/20 hover:border-orange-400/50 mx-2 sm:mx-0 rounded-3xl group">
        {/* Card background decoration */}
        <div className="absolute inset-0 bg-gradient-to-br from-orange-50/30 via-transparent to-yellow-50/30 dark:from-orange-900/10 dark:via-transparent dark:to-yellow-900/10" />
        <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full blur-xl group-hover:blur-2xl transition-all duration-300" />
        
        <CardHeader className="relative z-10">
          <CardDescription className="text-lg leading-relaxed text-gray-600 dark:text-gray-300">
            🚀 A <strong className="bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent font-bold">DataTech Platform</strong> is a comprehensive technology platform designed to manage the complete data lifecycle, from collection and storage to management, processing, analysis, and visualization. These platforms integrate various tools and technologies to help organizations and users maximize the value of their data, especially in the digital era where data plays a crucial role in business decision-making and operations across multiple sectors. ✨
          </CardDescription>
        </CardHeader>
      </Card>

      {/* Features Grid with enhanced PancakeSwap styling */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 sm:gap-8 px-2 sm:px-0">
        {features.map((feature, index) => {
          const gradients = [
            'from-orange-500 to-yellow-500',
            'from-blue-500 to-cyan-500',
            'from-purple-500 to-pink-500',
            'from-green-500 to-emerald-500',
            'from-red-500 to-orange-500',
            'from-indigo-500 to-purple-500'
          ];
          const bgGradients = [
            'from-orange-400/10 to-yellow-400/10',
            'from-blue-400/10 to-cyan-400/10',
            'from-purple-400/10 to-pink-400/10',
            'from-green-400/10 to-emerald-400/10',
            'from-red-400/10 to-orange-400/10',
            'from-indigo-400/10 to-purple-400/10'
          ];
          
          return (
            <Card 
              key={feature.id} 
              className="relative group hover:shadow-2xl transition-all duration-500 bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border border-orange-200/30 dark:border-orange-400/20 hover:border-orange-400/50 rounded-2xl overflow-hidden hover:scale-105"
              style={{ animationDelay: `${index * 0.1}s` }}
            >
              {/* Card background decoration */}
              <div className={`absolute inset-0 bg-gradient-to-br ${bgGradients[index]} opacity-50 group-hover:opacity-70 transition-opacity duration-300`} />
              <div className={`absolute top-0 right-0 w-20 h-20 bg-gradient-to-br ${bgGradients[index]} rounded-full blur-xl group-hover:blur-2xl transition-all duration-300`} />
              
              <CardHeader className="relative z-10">
                <h3 className={`text-xl sm:text-2xl font-bold bg-gradient-to-r ${gradients[index]} bg-clip-text text-transparent transition-all duration-300`}>
                  {feature.title}
                </h3>
              </CardHeader>
              <CardContent className="relative z-10">
                <p className="text-gray-600 dark:text-gray-300 mb-4 leading-relaxed">{feature.description}</p>
                <p className="text-sm text-gray-500 dark:text-gray-400 leading-relaxed">{feature.details}</p>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Benefits Section with PancakeSwap styling */}
      <Card className="relative mt-8 bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border border-orange-200/50 dark:border-orange-400/20 mx-2 sm:mx-0 rounded-3xl overflow-hidden group">
        {/* Card background decoration */}
        <div className="absolute inset-0 bg-gradient-to-br from-green-50/30 via-transparent to-emerald-50/30 dark:from-green-900/10 dark:via-transparent dark:to-emerald-900/10" />
        <div className="absolute bottom-0 left-0 w-40 h-40 bg-gradient-to-br from-green-400/10 to-emerald-400/10 rounded-full blur-2xl" />
        
        <CardHeader className="relative z-10">
          <h3 className="text-2xl sm:text-3xl font-bold bg-gradient-to-r from-green-500 to-emerald-500 bg-clip-text text-transparent">
            🎯 Benefits
          </h3>
        </CardHeader>
        <CardContent className="relative z-10">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
            <div className="space-y-4">
              <p className="flex items-center text-gray-600 dark:text-gray-300 font-medium">
                <span className="mr-3 text-green-500 text-xl">✅</span>
                Enable accurate and efficient data-driven decisions
              </p>
              <p className="flex items-center text-gray-600 dark:text-gray-300 font-medium">
                <span className="mr-3 text-green-500 text-xl">⚡</span>
                Increase speed in accessing and processing big data
              </p>
              <p className="flex items-center text-gray-600 dark:text-gray-300 font-medium">
                <span className="mr-3 text-green-500 text-xl">🔒</span>
                Improve data management organization and security
              </p>
            </div>
            <div className="space-y-4">
              <p className="flex items-center text-gray-600 dark:text-gray-300 font-medium">
                <span className="mr-3 text-green-500 text-xl">💰</span>
                Reduce costs through cloud systems and scalable storage
              </p>
              <p className="flex items-center text-gray-600 dark:text-gray-300 font-medium">
                <span className="mr-3 text-green-500 text-xl">🤝</span>
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
