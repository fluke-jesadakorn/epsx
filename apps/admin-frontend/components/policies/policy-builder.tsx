/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any */
'use client';

import React from 'react';

import { usePolicyBuilder } from './hooks/use-policy-builder';
import {
  PolicyBuilderHeader,
  TemplatesPanel,
  PolicyConfiguration,
  TargetActions,
  ConditionsBuilder,
  ActionsResponses,
  TestResults,
} from './policy-builder-sections';

export default function PolicyBuilder() {
  const {
    formData,
    setFormData,
    templates,
    showTemplates,
    setShowTemplates,
    testResults,
    saving,
    addCondition,
    removeCondition,
    updateCondition,
    addTargetAction,
    removeTargetAction,
    handleTestPolicy,
    handleSavePolicy,
  } = usePolicyBuilder();

  return (
    <div className="space-y-6 sm:space-y-8">
      <PolicyBuilderHeader
        showTemplates={showTemplates}
        setShowTemplates={setShowTemplates}
        onTest={handleTestPolicy}
        onSave={handleSavePolicy}
        saving={saving}
        formData={formData}
      />

      {showTemplates && (
        <TemplatesPanel
          templates={templates}
          onClose={() => setShowTemplates(false)}
        />
      )}

      <PolicyConfiguration
        formData={formData}
        setFormData={setFormData}
      />

      <TargetActions
        actions={formData.target_actions}
        onAdd={addTargetAction}
        onRemove={removeTargetAction}
      />

      <ConditionsBuilder
        formData={formData}
        setFormData={setFormData}
        onAdd={addCondition}
        onRemove={removeCondition}
        onUpdate={updateCondition}
      />

      <ActionsResponses
        formData={formData}
        setFormData={setFormData}
      />

      <TestResults testResults={testResults} />
    </div>
  );
}