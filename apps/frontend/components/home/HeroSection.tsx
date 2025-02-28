"use client";

import React from "react";
import type { CSSProperties } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { LineChart, Share2 } from "lucide-react";

interface HeroSectionProps {
  style?: CSSProperties;
  className?: string;
}

const HeroSection: React.FC<HeroSectionProps> = ({ style, className }) => {
  return (
    <div 
      className={`relative w-full flex items-center justify-center overflow-hidden ${className || ''}`}
      style={style}
    >
      {/* Background gradients */}
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,_#2196f3_0%,_transparent_25%)]" />
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,_#9c27b0_0%,_transparent_25%)]" />
      
      <div className="relative text-center space-y-8 max-w-3xl mx-auto px-4">
        <div className="space-y-4">
          <h1 className="text-5xl md:text-6xl font-bold">
            Track
            <span className="bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent mx-2">
              EPS Growth
            </span>
            Rankings
          </h1>
          <p className="text-xl md:text-2xl text-muted-foreground">
            Unlock deeper insights and make smarter investment decisions with real-time EPS tracking and advanced analytics
          </p>
        </div>

        <div className="flex flex-wrap justify-center gap-4">
          <Link href="/ranking">
            <Button 
              size="lg"
              className="bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 text-white rounded-full h-12 px-8 font-semibold"
            >
              <LineChart className="w-5 h-5 mr-2" />
              View Rankings
            </Button>
          </Link>
          <Button 
            variant="outline" 
            size="lg"
            className="rounded-full h-12 px-8 font-semibold hover:bg-primary/10"
          >
            <Share2 className="w-5 h-5 mr-2" />
            Share EPSx
          </Button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mt-12">
          {[
            { label: 'Active Users', value: '10K+' },
            { label: 'Markets', value: '3+' },
            { label: 'Companies Tracked', value: '1000+' },
          ].map((stat, i) => (
            <div key={i} className="bg-card/50 backdrop-blur-sm rounded-2xl p-6 border">
              <div className="text-3xl font-bold bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
                {stat.value}
              </div>
              <div className="text-muted-foreground mt-1">{stat.label}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default HeroSection;
