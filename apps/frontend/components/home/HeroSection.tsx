"use client";

import React from "react";
import type { CSSProperties } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { LineChart, Share2 } from "lucide-react";
import { toast } from "sonner";

interface HeroSectionProps {
  style?: CSSProperties;
  className?: string;
}

const HeroSection: React.FC<HeroSectionProps> = ({ style, className }) => {
  const handleShare = () => {
    const url = window.location.href;
    navigator.clipboard.writeText(url).then(() => {
      toast.success("URL copied to clipboard!");
    });
  };

  return (
    <div
      className={`w-full min-h-[85vh] flex items-center justify-center overflow-hidden ${className || ""}`}
      style={style}
    >

      <div className="relative text-center space-y-12 max-w-4xl mx-auto px-6 py-16 sm:px-8">
        <div className="space-y-6">
          <div className="inline-block animate-fade-in">
            <h1 className="text-5xl md:text-6xl lg:text-7xl font-bold">
              Track
              <span className="bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent mx-2 animate-gradient">
                EPS Growth
              </span>
              Rankings
            </h1>
          </div>
          <p className="text-xl md:text-2xl font-medium text-foreground/90 max-w-2xl mx-auto leading-relaxed animate-fade-in-delayed">
            Unlock deeper insights and optimize data center performance with real-time analytics and advanced data tracking systems for smarter operational decisions
          </p>
        </div>

        <div className="flex flex-wrap justify-center gap-6 animate-fade-in-delayed-2">
          <Link href="/ranking" className="cursor-pointer">
            <Button
              size="lg"
              className="cursor-pointer bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 hover:from-blue-600 hover:via-purple-600 hover:to-pink-600 text-white rounded-full h-14 px-10 font-semibold text-lg shadow-lg hover:shadow-xl transition-all duration-300 hover:scale-105"
            >
              <LineChart className="w-5 h-5 mr-2" />
              View Rankings
            </Button>
          </Link>
          <Button
            variant="outline"
            size="lg"
            className="rounded-full h-14 px-10 font-semibold hover:bg-primary/10 cursor-pointer text-lg border-2 hover:border-blue-500/50 transition-all duration-300 hover:scale-105"
            onClick={handleShare}
          >
            <Share2 className="w-5 h-5 mr-2" />
            Share EPSx
          </Button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mt-16 animate-fade-in-delayed-3">
          {[
            { label: "Ready for Users", value: "10K+" },
            { label: "Markets", value: "40+" },
            { label: "Companies Tracked", value: "10000+" },
          ].map((stat, i) => (
            <div
              key={i}
              className="bg-card/80 backdrop-blur-sm rounded-2xl p-8 border border-blue-500/20 hover:border-blue-500/30 transition-all duration-300 hover:scale-105 hover:shadow-lg group"
            >
              <div className="text-3xl font-bold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent group-hover:animate-gradient">
                {stat.value}
              </div>
              <div className="text-foreground/80 mt-1 font-medium">{stat.label}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default HeroSection;
