"use client";

import React from "react";

import { Card, CardHeader, CardContent, CardDescription } from "@/components/ui/card";

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
    <div className="flex flex-col gap-8 w-full p-4 sm:p-6 lg:p-8 max-w-7xl mx-auto my-8 sm:my-16">
      {/* Section Header */}
      <div className="text-center space-y-4 mb-6 px-4">
        <h2 className="text-4xl font-bold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent">
          DataTech Platform
        </h2>
        <div className="w-32 h-1 bg-gradient-to-r from-blue-500 to-purple-500 mx-auto rounded-full" />
      </div>

      {/* Overview Card */}
      <Card className="overflow-hidden transition-all duration-300 hover:shadow-xl bg-gradient-to-br from-card via-card/80 to-card/50 border border-blue-500/10 hover:border-blue-500/30 mx-2 sm:mx-0">
        <CardHeader>
          <CardDescription className="text-lg text-muted-foreground">
            A <strong className="text-primary">DataTech Platform</strong> is a comprehensive technology platform designed to manage the complete data lifecycle, from collection and storage to management, processing, analysis, and visualization. These platforms integrate various tools and technologies to help organizations and users maximize the value of their data, especially in the digital era where data plays a crucial role in business decision-making and operations across multiple sectors.
          </CardDescription>
        </CardHeader>
      </Card>

      {/* Features Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 px-2 sm:px-0">
        {features.map((feature) => (
          <Card key={feature.id} className="group hover:shadow-xl transition-all duration-300 border-blue-500/10 hover:border-blue-500/30">
            <CardHeader>
              <h3 className="text-xl font-semibold bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent group-hover:from-purple-500 group-hover:to-pink-500 transition-all duration-300">
                {feature.title}
              </h3>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground mb-4">{feature.description}</p>
              <p className="text-sm text-muted-foreground/80">{feature.details}</p>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Benefits Section */}
      <Card className="mt-8 bg-gradient-to-br from-card via-card/80 to-card/50 border-blue-500/10 mx-2 sm:mx-0">
        <CardHeader>
          <h3 className="text-2xl font-semibold bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
            Benefits
          </h3>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-2">
              <p className="flex items-center text-muted-foreground">
                <span className="mr-2">•</span>
                Enable accurate and efficient data-driven decisions
              </p>
              <p className="flex items-center text-muted-foreground">
                <span className="mr-2">•</span>
                Increase speed in accessing and processing big data
              </p>
              <p className="flex items-center text-muted-foreground">
                <span className="mr-2">•</span>
                Improve data management organization and security
              </p>
            </div>
            <div className="space-y-2">
              <p className="flex items-center text-muted-foreground">
                <span className="mr-2">•</span>
                Reduce costs through cloud systems and scalable storage
              </p>
              <p className="flex items-center text-muted-foreground">
                <span className="mr-2">•</span>
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
