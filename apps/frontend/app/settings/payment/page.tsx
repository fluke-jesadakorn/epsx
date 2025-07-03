'use client';

import { PaymentStatusSection } from '@/components/sections/payment/PaymentStatusSection';
import { SelectPackageSection } from '@/components/sections/payment/SelectPackageSection';
import { Card, CardHeader, CardContent } from '@/components/ui/card';

export default function PaymentSettingsPage() {
	return (
		<div className="max-w-3xl mx-auto p-6">
			{/* Page Header */}
			<div className="mb-8 text-center">
				<h1 className="text-3xl md:text-4xl font-bold tracking-tight text-primary mb-2 flex items-center justify-center gap-2">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						className="h-8 w-8 text-primary"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							strokeLinecap="round"
							strokeLinejoin="round"
							strokeWidth={2}
							d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
					Payment Settings
				</h1>
				<p className="text-muted-foreground text-base max-w-xl mx-auto">
					Manage your payment information and subscription details
				</p>
			</div>

			{/* Payment Status Section */}
			<PaymentStatusSection className="mb-8" />

			{/* Package Selection */}
			<Card className="transition-shadow hover:shadow-lg border bg-background/80 mb-8">
				<CardHeader className="flex flex-row items-center gap-2 pb-2">
					<span className="bg-primary/10 rounded-full p-2">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							className="h-6 w-6 text-primary"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								strokeLinecap="round"
								strokeLinejoin="round"
								strokeWidth={2}
								d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
							/>
						</svg>
					</span>
					<h2 className="text-xl font-semibold">Select Package</h2>
				</CardHeader>
				<CardContent className="p-0">
					<SelectPackageSection showTitle={false} />
				</CardContent>
			</Card>

		</div>
	);
}
