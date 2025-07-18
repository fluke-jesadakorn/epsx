'use client';

import { useState, useEffect } from 'react';
import { X, Save, Search, AlertCircle, Code, FileText, Plus, Trash2 } from 'lucide-react';
import type { Policy, Group } from '@/types/admin/iam';
import { POLICY_TEMPLATES } from '@/types/admin/iam';

/**
 * Policy Management Modal
 * AWS IAM-style policy creation and editing with JSON document editor
 */
export function PolicyModal({ 
  policy, 
  onClose, 
  onSave 
}: { 
  policy: Policy | null; 
  onClose: () => void; 
  onSave: () => void; 
}) {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    document: '',
    version: '1.0'
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [jsonError, setJsonError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'visual' | 'json'>('visual');
  
  // Visual editor state
  const [statements, setStatements] = useState([
    {
      effect: 'Allow',
      actions: [''],
      resources: ['*'],
      conditions: {}
    }
  ]);

  const defaultPolicyTemplates = {
    'Bronze': {
      "Version": "2024-01-01",
      "Statement": [{
        "Effect": "Allow",
        "Action": ["stock:rankings:read"],
        "Resource": "*",
        "Condition": {
          "NumericLessThan": {
            "stock:rankings:count": 10
          }
        }
      }]
    },
    'Silver': {
      "Version": "2024-01-01",
      "Statement": [{
        "Effect": "Allow",
        "Action": [
          "stock:rankings:read",
          "stock:analytics:analyze"
        ],
        "Resource": "*",
        "Condition": {
          "NumericLessThan": {
            "stock:rankings:count": 25
          }
        }
      }]
    },
    'Gold': {
      "Version": "2024-01-01",
      "Statement": [{
        "Effect": "Allow",
        "Action": [
          "stock:rankings:read",
          "stock:analytics:analyze",
          "stock:data:export"
        ],
        "Resource": "*",
        "Condition": {
          "NumericLessThan": {
            "stock:rankings:count": 50
          }
        }
      }]
    },
    'Platinum': {
      "Version": "2024-01-01",
      "Statement": [{
        "Effect": "Allow",
        "Action": [
          "stock:rankings:read",
          "stock:analytics:analyze",
          "stock:data:export",
          "stock:screener:screen"
        ],
        "Resource": "*",
        "Condition": {
          "NumericLessThan": {
            "stock:rankings:count": 100
          }
        }
      }]
    },
    'Admin': {
      "Version": "2024-01-01",
      "Statement": [{
        "Effect": "Allow",
        "Action": ["*"],
        "Resource": "*"
      }]
    }
  };

  useEffect(() => {
    if (policy) {
      setFormData({
        name: policy.name,
        description: policy.description,
        document: JSON.stringify(policy.policyDocument, null, 2),
        version: '1.0' // Default version since it's not in the Policy type
      });
    }
  }, [policy]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setJsonError(null);

    try {
      // Validate form
      if (!formData.name || !formData.description) {
        throw new Error('Name and description are required');
      }

      // Validate JSON
      let parsedDocument;
      try {
        parsedDocument = JSON.parse(formData.document);
      } catch (jsonErr) {
        setJsonError('Invalid JSON format');
        return;
      }

      // Validate policy structure
      if (!parsedDocument.Version || !parsedDocument.Statement) {
        throw new Error('Policy must have Version and Statement fields');
      }

      // Mock API call - replace with actual implementation
      console.log('Saving policy:', {
        ...formData,
        document: parsedDocument
      });
      
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      onSave();
    } catch (err: any) {
      setError(err.message || 'Failed to save policy');
    } finally {
      setLoading(false);
    }
  };

  const handleTemplateSelect = (templateName: string) => {
    const template = defaultPolicyTemplates[templateName as keyof typeof defaultPolicyTemplates];
    if (template) {
      setFormData(prev => ({
        ...prev,
        document: JSON.stringify(template, null, 2)
      }));
    }
  };

  const addStatement = () => {
    setStatements(prev => [...prev, {
      effect: 'Allow',
      actions: [''],
      resources: ['*'],
      conditions: {}
    }]);
  };

  const removeStatement = (index: number) => {
    setStatements(prev => prev.filter((_, i) => i !== index));
  };

  const updateStatement = (index: number, field: string, value: any) => {
    setStatements(prev => prev.map((stmt, i) => 
      i === index ? { ...stmt, [field]: value } : stmt
    ));
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-10 mx-auto p-5 border w-full max-w-6xl shadow-lg rounded-md bg-white dark:bg-gray-800">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            {policy ? 'Edit Policy' : 'Create New Policy'}
          </h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <X className="h-6 w-6" />
          </button>
        </div>

        {error && (
          <div className="mb-4 bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-800 rounded-md p-4">
            <div className="flex">
              <AlertCircle className="h-5 w-5 text-red-400 mr-2" />
              <p className="text-sm text-red-800 dark:text-red-200">{error}</p>
            </div>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Policy Name *
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Version
              </label>
              <input
                type="text"
                value={formData.version}
                onChange={(e) => setFormData(prev => ({ ...prev, version: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Description *
            </label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              required
            />
          </div>

          {/* Policy Templates */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Use Template (Optional)
            </label>
            <div className="flex flex-wrap gap-2">
              {Object.keys(defaultPolicyTemplates).map((templateName) => (
                <button
                  key={templateName}
                  type="button"
                  onClick={() => handleTemplateSelect(templateName)}
                  className="px-3 py-1 text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-md hover:bg-gray-200 dark:hover:bg-gray-600"
                >
                  {templateName}
                </button>
              ))}
            </div>
          </div>

          {/* Policy Document Editor */}
          <div>
            <div className="flex items-center justify-between mb-3">
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Policy Document *
              </label>
              <div className="flex rounded-md shadow-sm">
                <button
                  type="button"
                  onClick={() => setActiveTab('visual')}
                  className={`px-3 py-1 text-sm font-medium rounded-l-md border ${
                    activeTab === 'visual'
                      ? 'bg-blue-50 border-blue-500 text-blue-700'
                      : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'
                  }`}
                >
                  <FileText className="h-4 w-4 inline mr-1" />
                  Visual
                </button>
                <button
                  type="button"
                  onClick={() => setActiveTab('json')}
                  className={`px-3 py-1 text-sm font-medium rounded-r-md border-t border-r border-b ${
                    activeTab === 'json'
                      ? 'bg-blue-50 border-blue-500 text-blue-700'
                      : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'
                  }`}
                >
                  <Code className="h-4 w-4 inline mr-1" />
                  JSON
                </button>
              </div>
            </div>

            {activeTab === 'json' ? (
              <div>
                <textarea
                  value={formData.document}
                  onChange={(e) => setFormData(prev => ({ ...prev, document: e.target.value }))}
                  rows={15}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono text-sm"
                  placeholder="Enter policy document JSON..."
                />
                {jsonError && (
                  <p className="mt-2 text-sm text-red-600 dark:text-red-400">{jsonError}</p>
                )}
              </div>
            ) : (
              <div className="border border-gray-300 dark:border-gray-600 rounded-md p-4 bg-gray-50 dark:bg-gray-700">
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                  Visual editor coming soon. Please use JSON editor for now.
                </p>
                <div className="space-y-4">
                  {statements.map((statement, index) => (
                    <div key={index} className="bg-white dark:bg-gray-800 p-4 rounded-md border border-gray-200 dark:border-gray-600">
                      <div className="flex justify-between items-center mb-3">
                        <h4 className="text-sm font-medium text-gray-900 dark:text-white">
                          Statement {index + 1}
                        </h4>
                        <button
                          type="button"
                          onClick={() => removeStatement(index)}
                          className="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-200"
                        >
                          <Trash2 className="h-4 w-4" />
                        </button>
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                          <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Effect
                          </label>
                          <select
                            value={statement.effect}
                            onChange={(e) => updateStatement(index, 'effect', e.target.value)}
                            className="w-full px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                          >
                            <option value="Allow">Allow</option>
                            <option value="Deny">Deny</option>
                          </select>
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Actions
                          </label>
                          <input
                            type="text"
                            value={statement.actions.join(', ')}
                            onChange={(e) => updateStatement(index, 'actions', e.target.value.split(', '))}
                            className="w-full px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                            placeholder="e.g., stock:rankings:read"
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                  <button
                    type="button"
                    onClick={addStatement}
                    className="w-full px-4 py-2 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-md text-sm text-gray-600 dark:text-gray-400 hover:border-gray-400 dark:hover:border-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
                  >
                    <Plus className="h-4 w-4 inline mr-2" />
                    Add Statement
                  </button>
                </div>
              </div>
            )}
          </div>

          <div className="flex justify-end space-x-3 pt-6 border-t border-gray-200 dark:border-gray-700">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
            >
              {loading ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                  Saving...
                </>
              ) : (
                <>
                  <Save className="h-4 w-4 mr-2" />
                  {policy ? 'Update Policy' : 'Create Policy'}
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

/**
 * Group Management Modal
 * AWS IAM-style group creation and editing
 */
export function GroupModal({ 
  group, 
  onClose, 
  onSave 
}: { 
  group: Group | null; 
  onClose: () => void; 
  onSave: () => void; 
}) {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    attachedPolicies: [] as string[]
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [availablePolicies, setAvailablePolicies] = useState<Policy[]>([]);
  const [policySearch, setPolicySearch] = useState('');

  useEffect(() => {
    if (group) {
      setFormData({
        name: group.name,
        description: group.description,
        attachedPolicies: group.attachedPolicies
      });
    }
    loadAvailablePolicies();
  }, [group]);

  const loadAvailablePolicies = async () => {
    try {
      // Mock data - replace with actual API calls
      setAvailablePolicies([
        { id: '1', name: 'BronzePolicy', description: 'Basic access policy', policyDocument: POLICY_TEMPLATES.Bronze, arn: '', path: '/', isAttachable: true, attachmentCount: 0, createdAt: '2024-01-01', updatedAt: '2024-01-01' },
        { id: '2', name: 'SilverPolicy', description: 'Premium access policy', policyDocument: POLICY_TEMPLATES.Silver, arn: '', path: '/', isAttachable: true, attachmentCount: 0, createdAt: '2024-01-01', updatedAt: '2024-01-01' },
        { id: '3', name: 'GoldPolicy', description: 'Advanced access policy', policyDocument: POLICY_TEMPLATES.Gold, arn: '', path: '/', isAttachable: true, attachmentCount: 0, createdAt: '2024-01-01', updatedAt: '2024-01-01' },
        { id: '4', name: 'PlatinumPolicy', description: 'Full access policy', policyDocument: POLICY_TEMPLATES.Platinum, arn: '', path: '/', isAttachable: true, attachmentCount: 0, createdAt: '2024-01-01', updatedAt: '2024-01-01' },
        { id: '5', name: 'AdminPolicy', description: 'Administrative access policy', policyDocument: POLICY_TEMPLATES.Admin, arn: '', path: '/', isAttachable: true, attachmentCount: 0, createdAt: '2024-01-01', updatedAt: '2024-01-01' }
      ]);
    } catch (err) {
      console.error('Failed to load policies:', err);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      // Validate form
      if (!formData.name || !formData.description) {
        throw new Error('Name and description are required');
      }

      // Mock API call - replace with actual implementation
      console.log('Saving group:', formData);
      
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      onSave();
    } catch (err: any) {
      setError(err.message || 'Failed to save group');
    } finally {
      setLoading(false);
    }
  };

  const handlePolicyToggle = (policyId: string) => {
    setFormData(prev => ({
      ...prev,
      attachedPolicies: prev.attachedPolicies.includes(policyId)
        ? prev.attachedPolicies.filter(p => p !== policyId)
        : [...prev.attachedPolicies, policyId]
    }));
  };

  const filteredPolicies = availablePolicies.filter(policy =>
    policy.name.toLowerCase().includes(policySearch.toLowerCase()) ||
    policy.description.toLowerCase().includes(policySearch.toLowerCase())
  );

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-full max-w-2xl shadow-lg rounded-md bg-white dark:bg-gray-800">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            {group ? 'Edit Group' : 'Create New Group'}
          </h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <X className="h-6 w-6" />
          </button>
        </div>

        {error && (
          <div className="mb-4 bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-800 rounded-md p-4">
            <div className="flex">
              <AlertCircle className="h-5 w-5 text-red-400 mr-2" />
              <p className="text-sm text-red-800 dark:text-red-200">{error}</p>
            </div>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Group Name *
            </label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Description *
            </label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Attached Policies
            </label>
            
            <div className="relative mb-3">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
              <input
                type="text"
                placeholder="Search policies..."
                value={policySearch}
                onChange={(e) => setPolicySearch(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>

            <div className="max-h-40 overflow-y-auto border border-gray-300 dark:border-gray-600 rounded-md">
              {filteredPolicies.map((policy) => (
                <div key={policy.id} className="p-3 hover:bg-gray-50 dark:hover:bg-gray-700 border-b border-gray-200 dark:border-gray-600 last:border-b-0">
                  <div className="flex items-center">
                    <input
                      id={`policy-${policy.id}`}
                      type="checkbox"
                      checked={formData.attachedPolicies.includes(policy.id)}
                      onChange={() => handlePolicyToggle(policy.id)}
                      className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                    />
                    <label htmlFor={`policy-${policy.id}`} className="ml-3 flex-1">
                      <div className="text-sm font-medium text-gray-900 dark:text-white">{policy.name}</div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">{policy.description}</div>
                    </label>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="flex justify-end space-x-3 pt-6 border-t border-gray-200 dark:border-gray-700">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
            >
              {loading ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                  Saving...
                </>
              ) : (
                <>
                  <Save className="h-4 w-4 mr-2" />
                  {group ? 'Update Group' : 'Create Group'}
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
