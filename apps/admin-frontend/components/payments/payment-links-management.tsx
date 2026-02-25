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
            <div className="relative max-w-7xl mx-auto">
                {/* Action Bar */}
                <div className="flex items-center gap-3 mb-6">
                    <ActionCards onCreateClick={() => setIsModalOpen(true)} onRefreshClick={() => void loadPaymentLinks()} />
                </div>

                <FilterSection
                    filterType={filterType}
                    onFilterTypeChange={setFilterType}
                    filterActive={filterActive}
                    onFilterActiveChange={setFilterActive}
                    onReset={resetFilters}
                />

                {error && (
                    <div className="relative overflow-hidden rounded-2xl bg-destructive/10 p-0.5 mb-6">
                        <div className="bg-destructive/5 rounded-2xl p-4 text-destructive border border-destructive/20">
                            {error}
                        </div>
                    </div>
                )}

                {/* Payment Links Table */}
                <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
                    <div className="h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]" />
                    <div>
                        <div className="p-4 sm:p-6 lg:p-8">
                            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
                                <h2 className="text-xs font-bold text-[#7645d9] uppercase tracking-[0.2em]">
                                    All Payment Links
                                </h2>
                                <div className="px-3 py-1 bg-muted/50 rounded-full border border-border/40 text-xs font-bold text-muted-foreground">{paymentLinks.length} links</div>
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
