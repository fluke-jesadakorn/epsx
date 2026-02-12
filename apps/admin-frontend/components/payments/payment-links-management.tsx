'use client';

import React, { useCallback, useState } from 'react';
import {
    ActionCards,
    CreatePaymentLinkModal,
    EmptyState,
    FilterSection,
    LoadingState,
    PaymentLinksMobileCards,
    PaymentLinksTable,
} from './payment-links-ui';
import type {
    CreatePaymentLinkForm} from './payment-links-hooks';
import {
    usePaymentLinkActions,
    usePaymentLinkFilters,
    usePaymentLinkForm,
    usePaymentLinkFormatting,
    usePaymentLinks,
} from './payment-links-hooks';

export function PaymentLinksManagement() {
    const [isModalOpen, setIsModalOpen] = useState(false);

    const { filterType, setFilterType, filterActive, setFilterActive, resetFilters } = usePaymentLinkFilters();
    const { paymentLinks, setPaymentLinks, loading, error, loadPaymentLinks } = usePaymentLinks({
        filterType,
        filterActive,
    });

    const { form, setForm, formLoading, formError, handleCreateLink, resetForm } = usePaymentLinkForm({
        onSuccess: () => {
            setIsModalOpen(false);
            void loadPaymentLinks();
        },
    });

    const { copiedSlug, handleCopyUrl, handleDeleteLink } = usePaymentLinkActions();
    const { isExpired, formatDate } = usePaymentLinkFormatting();

    const handleFormChange = useCallback(<K extends keyof CreatePaymentLinkForm>(key: K, value: CreatePaymentLinkForm[K]) => {
        setForm((prev) => ({ ...prev, [key]: value }));
    }, [setForm]);

    const handleDeleteWithRefresh = useCallback(
        async (id: string) => {
            await handleDeleteLink(id, () => {
                setPaymentLinks((prev) =>
                    prev.map((link) => (link.id === id ? { ...link, is_active: false, is_usable: false } : link))
                );
            });
        },
        [handleDeleteLink, setPaymentLinks]
    );

    if (loading) {
        return <LoadingState />;
    }

    return (
        <div className="space-y-6 sm:space-y-8">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-xl" />
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-blue-400/20 to-indigo-500/20 rounded-full blur-lg" />
            </div>

            <div className="relative max-w-7xl mx-auto">
                {/* Hero Section */}
                <div className="text-center mb-8 sm:mb-12">
                    <div className="relative inline-block">
                        <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent mb-4">
                            🔗 Payment Links
                        </h1>
                        <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary/20 rounded-full animate-ping" />
                    </div>
                    <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
                        Create and manage dynamic payment links for plans, products, and campaigns
                    </p>
                </div>

                <ActionCards onCreateClick={() => setIsModalOpen(true)} onRefreshClick={() => void loadPaymentLinks()} />

                <FilterSection
                    filterType={filterType}
                    onFilterTypeChange={setFilterType}
                    filterActive={filterActive}
                    onFilterActiveChange={setFilterActive}
                    onReset={resetFilters}
                />

                {error && (
                    <div className="relative overflow-hidden rounded-2xl bg-destructive/10 p-0.5 mb-6">
                        <div className="bg-destructive/5 backdrop-blur-xl rounded-2xl p-4 text-destructive border border-destructive/20">
                            {error}
                        </div>
                    </div>
                )}

                {/* Payment Links Table */}
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5">
                    <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl overflow-hidden border border-border/50">
                        <div className="p-4 sm:p-6 lg:p-8">
                            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
                                <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                                    All Payment Links
                                </h2>
                                <div className="text-sm text-muted-foreground">{paymentLinks.length} links</div>
                            </div>

                            {paymentLinks.length === 0 ? (
                                <EmptyState />
                            ) : (
                                <>
                                    <PaymentLinksMobileCards
                                        links={paymentLinks}
                                        copiedSlug={copiedSlug}
                                        onCopyUrl={handleCopyUrl}
                                        onDelete={handleDeleteWithRefresh}
                                        isExpired={isExpired}
                                    />

                                    <PaymentLinksTable
                                        links={paymentLinks}
                                        copiedSlug={copiedSlug}
                                        onCopyUrl={handleCopyUrl}
                                        onDelete={handleDeleteWithRefresh}
                                        formatDate={formatDate}
                                        isExpired={isExpired}
                                    />
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            <CreatePaymentLinkModal
                isOpen={isModalOpen}
                onClose={() => {
                    setIsModalOpen(false);
                    resetForm();
                }}
                form={form}
                onFormChange={handleFormChange}
                onSubmit={handleCreateLink}
                isLoading={formLoading}
                error={formError}
            />
        </div>
    );
}
