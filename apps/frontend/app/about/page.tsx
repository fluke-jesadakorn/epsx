import type { Metadata } from 'next';
import DataTechSection from '@/components/about/data-tech-section';

export const metadata: Metadata = {
  title: 'About Us - EPSX Analytics Platform',
  description: 'Learn about EPSX DataTech Platform - comprehensive technology platform designed to manage the complete data lifecycle, from collection and storage to analysis and visualization.',
  keywords: 'EPSX, DataTech Platform, data analytics, business intelligence, data management',
};

export default function AboutPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
        <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 blur-3xl" />
        <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-3xl" />
        <div className="absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-emerald-400/15 blur-3xl" />

        {/* Mesh gradient overlays for depth */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)]" />

        {/* Decorative geometric shapes */}
        <div className="absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
        <div className="absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />
      </div>

      {/* Main content */}
      <div className="relative z-10">
        {/* Page Header */}
        <div className="container mx-auto px-4 pt-16 pb-8">
          <div className="text-center">
            <h1 className="text-5xl sm:text-6xl font-bold bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent mb-4">
              About EPSX
            </h1>
            <p className="mx-auto max-w-3xl text-lg leading-relaxed text-gray-600 dark:text-gray-300">
              Empowering businesses with advanced data analytics and comprehensive platform solutions
            </p>
            <div className="w-40 h-1 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 mx-auto rounded-full mt-6" />
          </div>
        </div>

        {/* DataTech Platform Section */}
        <DataTechSection />

        {/* Additional About Content */}
        <div className="container mx-auto px-4 py-16">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
            {/* Mission Section */}
            <div className="relative">
              <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl" />
              <div className="relative bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border border-blue-200/50 dark:border-blue-400/20 rounded-3xl p-8 shadow-2xl">
                <h2 className="text-3xl font-bold bg-gradient-to-r from-blue-500 to-cyan-500 bg-clip-text text-transparent mb-4">
                  Our Mission
                </h2>
                <p className="text-gray-600 dark:text-gray-300 leading-relaxed">
                  At EPSX, we're dedicated to transforming how businesses interact with their data. 
                  Our mission is to democratize advanced analytics and make powerful data insights 
                  accessible to organizations of all sizes, enabling smarter decisions and driving 
                  sustainable growth through innovative technology solutions.
                </p>
              </div>
            </div>

            {/* Vision Section */}
            <div className="relative">
              <div className="absolute -top-8 -right-8 h-16 w-16 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-xl" />
              <div className="relative bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border border-purple-200/50 dark:border-purple-400/20 rounded-3xl p-8 shadow-2xl">
                <h2 className="text-3xl font-bold bg-gradient-to-r from-purple-500 to-pink-500 bg-clip-text text-transparent mb-4">
                  Our Vision
                </h2>
                <p className="text-gray-600 dark:text-gray-300 leading-relaxed">
                  We envision a future where every business decision is powered by intelligent, 
                  real-time data insights. By building cutting-edge analytics platforms and 
                  fostering a data-driven culture, we aim to be the catalyst that helps 
                  organizations unlock their full potential and achieve extraordinary outcomes.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}