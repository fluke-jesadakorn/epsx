'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

interface SelectPackageProps {
  amount: string;
  currency: string;
  onAmountChange: (amount: string) => void;
  onCurrencyChange: (currency: string) => void;
  onNext: () => void;
}

export function SelectPackage({
  amount,
  currency,
  onAmountChange,
  onCurrencyChange,
  onNext,
}: SelectPackageProps) {
  return (
    <TooltipProvider>
    <div className="space-y-8 p-6 rounded-xl border bg-card shadow-sm">
      <div className="space-y-2">
        <h3 className="text-2xl font-bold text-primary">
          Step 1: Select Package
        </h3>
        <p className="text-muted-foreground">
          Choose the plan that best fits your needs
        </p>
      </div>
      <div className="space-y-6">
        <div className="space-y-10">
          <div className="space-y-4">
            <h4 className="text-lg font-semibold text-primary flex items-center gap-2">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                <circle cx="12" cy="7" r="4"></circle>
              </svg>
              Personal Plans
            </h4>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div
                className={`relative rounded-lg border-2 ${amount === '0' ? 'border-primary' : 'border-border'} transition-all duration-200 hover:border-primary/50`}
              >
                <Button
                  type="button"
                  variant={amount === '0' ? 'default' : 'outline'}
                  onClick={() => onAmountChange('0')}
                  className="w-full h-full min-h-[160px] flex flex-col gap-3 p-4 hover:scale-[1.02] transition-transform duration-200"
                  disabled={true}
                >
                  <span className="font-bold text-lg">Free Package</span>
                  <span className="text-sm text-muted-foreground">
                    Basic features for starting out
                  </span>
                  <div className="mt-auto">
                    <span className="text-2xl font-bold">$0</span>
                    <span className="text-sm text-muted-foreground">/month</span>
                  </div>
                  <div className="space-y-1 text-xs text-muted-foreground mt-2">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Limited API access</div>
                      </TooltipTrigger>
                      <TooltipContent>Up to 1,000 API calls per month</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Basic analytics</div>
                      </TooltipTrigger>
                      <TooltipContent>Market trends and basic portfolio tracking</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Community support</div>
                      </TooltipTrigger>
                      <TooltipContent>Access to community forums and documentation</TooltipContent>
                    </Tooltip>
                  </div>
                </Button>
                <div className="absolute -top-3 left-2 px-3 py-1 bg-muted text-xs font-medium rounded-full shadow-sm">
                  All Level
                </div>
              </div>
              <div
                className={`relative rounded-lg border-2 ${amount === '9.9' ? 'border-primary' : 'border-border'} transition-all duration-200 hover:border-primary/50`}
              >
                <Button
                  type="button"
                  variant={amount === '9.9' ? 'default' : 'outline'}
                  onClick={() => onAmountChange('9.9')}
                  className="w-full h-full min-h-[160px] flex flex-col gap-3 p-4 hover:scale-[1.02] transition-transform duration-200"
                >
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="font-bold text-lg cursor-help">PersonalX</span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Perfect for individual traders and investors</p>
                    </TooltipContent>
                  </Tooltip>
                  <span className="text-sm text-muted-foreground">
                    Enhanced features for individuals
                  </span>
                  <div className="mt-auto">
                    <span className="text-2xl font-bold">{currency} 9.9</span>
                    <span className="text-sm text-muted-foreground">/month</span>
                  </div>
                  <div className="space-y-1 text-xs text-muted-foreground mt-2">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Full API access</div>
                      </TooltipTrigger>
                      <TooltipContent>Unlimited API calls with standard rate limits</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Advanced analytics</div>
                      </TooltipTrigger>
                      <TooltipContent>Real-time market analysis and portfolio insights</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Priority support</div>
                      </TooltipTrigger>
                      <TooltipContent>Fast response time support via email</TooltipContent>
                    </Tooltip>
                  </div>
                </Button>
                <div className="absolute -top-3 left-2 px-3 py-1 bg-gradient-to-r from-primary to-primary/80 text-primary-foreground text-xs font-medium rounded-full shadow-md">
                  Most Popular
                </div>
              </div>
              <div
                className={`relative rounded-lg border-2 ${amount === '19.9' ? 'border-primary' : 'border-border'} transition-all duration-200 hover:border-primary/50`}
              >
                <Button
                  type="button"
                  variant={amount === '19.9' ? 'default' : 'outline'}
                  onClick={() => onAmountChange('19.9')}
                  className="w-full h-full min-h-[160px] flex flex-col gap-3 p-4 hover:scale-[1.02] transition-transform duration-200"
                >
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="font-bold text-lg cursor-help">ProfessionalY</span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Enhanced features for professional traders</p>
                    </TooltipContent>
                  </Tooltip>
                  <span className="text-sm text-muted-foreground">
                    Advanced tools for professionals
                  </span>
                  <div className="mt-auto">
                    <span className="text-2xl font-bold">{currency} 19.9</span>
                    <span className="text-sm text-muted-foreground">/month</span>
                  </div>
                  <div className="space-y-1 text-xs text-muted-foreground mt-2">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Real-time market analysis</div>
                      </TooltipTrigger>
                      <TooltipContent>Live market data and advanced technical indicators</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Custom alert system</div>
                      </TooltipTrigger>
                      <TooltipContent>Set custom price alerts and notifications</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• 24/7 dedicated support</div>
                      </TooltipTrigger>
                      <TooltipContent>Round-the-clock support via email and chat</TooltipContent>
                    </Tooltip>
                  </div>
                </Button>
                <div className="absolute -top-3 left-2 px-3 py-1 bg-orange-500 text-white text-xs font-medium rounded-full shadow-sm">
                  Best Value
                </div>
              </div>
              <div
                className={`relative rounded-lg border-2 ${amount === '29.9' ? 'border-primary' : 'border-border'} transition-all duration-200 hover:border-primary/50`}
              >
                <Button
                  type="button"
                  variant={amount === '29.9' ? 'default' : 'outline'}
                  onClick={() => onAmountChange('29.9')}
                  className="w-full h-full min-h-[160px] flex flex-col gap-3 p-4 hover:scale-[1.02] transition-transform duration-200"
                >
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="font-bold text-lg cursor-help">EnterpriseZ</span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Ultimate package for serious traders</p>
                    </TooltipContent>
                  </Tooltip>
                  <span className="text-sm text-muted-foreground">
                    Complete solution for power users
                  </span>
                  <div className="mt-auto">
                    <span className="text-2xl font-bold">{currency} 29.9</span>
                    <span className="text-sm text-muted-foreground">/month</span>
                  </div>
                  <div className="space-y-1 text-xs text-muted-foreground mt-2">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Unlimited API access</div>
                      </TooltipTrigger>
                      <TooltipContent>Unlimited API calls with higher rate limits</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Custom integrations</div>
                      </TooltipTrigger>
                      <TooltipContent>Custom API endpoints and integrations</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• VIP support & training</div>
                      </TooltipTrigger>
                      <TooltipContent>24/7 priority support and personalized training</TooltipContent>
                    </Tooltip>
                  </div>
                </Button>
              </div>
            </div>
          </div>
          <div className="space-y-6">
            <h4 className="text-lg font-semibold text-primary flex items-center gap-2">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M20 7h-9"></path>
                <path d="M14 17H5"></path>
                <circle cx="17" cy="17" r="3"></circle>
                <circle cx="7" cy="7" r="3"></circle>
              </svg>
              Business & API Plans
            </h4>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              <div
                className={`relative rounded-lg border-2 ${amount === '999' ? 'border-primary' : 'border-border'} transition-all duration-200 hover:border-primary/50`}
              >
                <Button
                  type="button"
                  variant={amount === '999' ? 'default' : 'outline'}
                  onClick={() => onAmountChange('999')}
                  className="w-full h-full min-h-[160px] flex flex-col gap-3 p-4 hover:scale-[1.02] transition-transform duration-200"
                >
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="font-bold text-lg cursor-help">API Personal</span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>API integration for personal projects</p>
                    </TooltipContent>
                  </Tooltip>
                  <span className="text-sm text-muted-foreground">
                    API access for individual developers
                  </span>
                  <div className="mt-auto">
                    <span className="text-2xl font-bold">{currency} 999</span>
                    <span className="text-sm text-muted-foreground">/month</span>
                  </div>
                  <div className="space-y-1 text-xs text-muted-foreground mt-2">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• 100k API calls/month</div>
                      </TooltipTrigger>
                      <TooltipContent>Up to 100,000 API calls per month with standard rate limits</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Developer support</div>
                      </TooltipTrigger>
                      <TooltipContent>Technical support and API documentation</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Testing environment</div>
                      </TooltipTrigger>
                      <TooltipContent>Sandbox environment for testing integrations</TooltipContent>
                    </Tooltip>
                  </div>
                </Button>
              </div>
              <div
                className={`relative rounded-lg border-2 ${amount === '2999' ? 'border-primary' : 'border-border'} transition-all duration-200 hover:border-primary/50`}
              >
                <Button
                  type="button"
                  variant={amount === '2999' ? 'default' : 'outline'}
                  onClick={() => onAmountChange('2999')}
                  className="w-full h-full min-h-[160px] flex flex-col gap-3 p-4 hover:scale-[1.02] transition-transform duration-200"
                >
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="font-bold text-lg cursor-help">API Company</span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Enterprise-grade API access with unlimited calls</p>
                    </TooltipContent>
                  </Tooltip>
                  <span className="text-sm text-muted-foreground">
                    Enterprise-grade API solution
                  </span>
                  <div className="mt-auto">
                    <span className="text-2xl font-bold">{currency} 2,999</span>
                    <span className="text-sm text-muted-foreground">/month</span>
                  </div>
                  <div className="space-y-1 text-xs text-muted-foreground mt-2">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Unlimited API calls</div>
                      </TooltipTrigger>
                      <TooltipContent>Unlimited API calls with premium rate limits</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Priority support</div>
                      </TooltipTrigger>
                      <TooltipContent>24/7 priority technical support</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Custom solutions</div>
                      </TooltipTrigger>
                      <TooltipContent>Custom API features and enterprise solutions</TooltipContent>
                    </Tooltip>
                  </div>
                </Button>
                <div className="absolute -top-3 left-2 px-3 py-1 bg-blue-500 text-white text-xs font-medium rounded-full shadow-sm">
                  Enterprise
                </div>
              </div>
              <div
                className={`relative rounded-lg border-2 ${amount === '999.1' ? 'border-primary' : 'border-border'} transition-all duration-200 hover:border-primary/50`}
              >
                <Button
                  type="button"
                  variant={amount === '999.1' ? 'default' : 'outline'}
                  onClick={() => onAmountChange('999.1')}
                  className="w-full h-full min-h-[160px] flex flex-col gap-3 p-4 hover:scale-[1.02] transition-transform duration-200"
                >
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="font-bold text-lg cursor-help">Company Plan</span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Complete solution for business teams</p>
                    </TooltipContent>
                  </Tooltip>
                  <span className="text-sm text-muted-foreground">
                    Complete business solution
                  </span>
                  <div className="mt-auto">
                    <span className="text-2xl font-bold">{currency} 999</span>
                    <span className="text-sm text-muted-foreground">/month</span>
                  </div>
                  <div className="space-y-1 text-xs text-muted-foreground mt-2">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Team collaboration</div>
                      </TooltipTrigger>
                      <TooltipContent>Multi-user access and team management features</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Advanced reporting</div>
                      </TooltipTrigger>
                      <TooltipContent>Customizable reports and analytics dashboard</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="cursor-help">• Dedicated account manager</div>
                      </TooltipTrigger>
                      <TooltipContent>Personal account manager for support and optimization</TooltipContent>
                    </Tooltip>
                  </div>
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="flex items-center justify-between p-6 bg-gradient-to-r from-muted/30 to-muted/10 rounded-xl space-x-4 border">
        <div className="flex items-center gap-2">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="16" x2="12" y2="12"></line>
            <line x1="12" y1="8" x2="12.01" y2="8"></line>
          </svg>
          <Label htmlFor="currency" className="font-semibold">
            Select Payment Currency
          </Label>
        </div>
        <Select value={currency} onValueChange={onCurrencyChange}>
          <SelectTrigger id="currency" className="bg-background border-2 w-[200px] transition-colors hover:border-primary">
            <SelectValue placeholder="Select currency" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="USDT">USDT</SelectItem>
          </SelectContent>
        </Select>
        <div className="flex items-start gap-2 text-sm text-orange-500">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
            <line x1="12" y1="9" x2="12" y2="13"></line>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
          <p>
            Currently, only USDT is supported for cryptocurrency payments. More
            payment options coming soon.
          </p>
        </div>
      </div>
      <Button
        onClick={onNext}
        className="w-full py-6 text-lg font-medium bg-gradient-to-r from-primary to-primary/90 hover:from-primary/90 hover:to-primary text-primary-foreground shadow-lg hover:shadow-xl transition-all duration-300 border-2 rounded-3xl bg-pink-100 dark:bg-gray-700 hover:border-primary"
      >
        Continue to Next Step
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="ml-2"
        >
          <line x1="5" y1="12" x2="19" y2="12"></line>
          <polyline points="12 5 19 12 12 19"></polyline>
        </svg>
      </Button>
    </div>
    </TooltipProvider>
  );
}
