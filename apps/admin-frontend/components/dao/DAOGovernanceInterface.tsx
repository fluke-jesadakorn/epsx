'use client'

/**
 * DAO Governance Interface - Admin Dashboard
 * Comprehensive DAO management and governance tracking for Web3 enterprise
 */

import { useState, useEffect } from 'react';
import { useAuth } from '@/lib/auth';

interface DAOMembership {
  id: string;
  wallet_address: string;
  dao_name: string;
  dao_contract: string;
  network: string;
  membership_type: 'token_holder' | 'nft_holder' | 'delegated' | 'multisig';
  governance_tokens: number;
  voting_power: number;
  joined_at: string;
  last_vote: string;
  delegation_status: 'none' | 'delegated_to' | 'delegated_from';
  delegated_to?: string;
  delegated_from?: string[];
}

interface GovernanceProposal {
  id: string;
  dao_name: string;
  proposal_id: string;
  title: string;
  description: string;
  proposer: string;
  status: 'active' | 'succeeded' | 'defeated' | 'expired' | 'executed';
  votes_for: number;
  votes_against: number;
  votes_abstain: number;
  total_votes: number;
  quorum_required: number;
  voting_deadline: string;
  execution_deadline?: string;
  network: string;
  proposal_url: string;
}

interface DAOAnalytics {
  total_daos_tracked: number;
  total_active_members: number;
  total_governance_tokens: number;
  active_proposals: number;
  completed_votes_24h: number;
  top_daos_by_members: Array<{
    dao_name: string;
    members: number;
    total_voting_power: number;
  }>;
  governance_activity: Array<{
    date: string;
    proposals_created: number;
    votes_cast: number;
  }>;
  network_distribution: Record<string, number>;
}

interface VotingHistory {
  id: string;
  wallet_address: string;
  dao_name: string;
  proposal_id: string;
  proposal_title: string;
  vote_choice: 'for' | 'against' | 'abstain';
  voting_power_used: number;
  voted_at: string;
  transaction_hash: string;
  network: string;
}

interface DelegationRecord {
  id: string;
  delegator: string;
  delegatee: string;
  dao_name: string;
  voting_power: number;
  delegated_at: string;
  is_active: boolean;
  network: string;
  transaction_hash: string;
}

export default function DAOGovernanceInterface() {
  const { user, canViewAnalytics } = useAuth();
  const canManageDAO = () => true; // Stubbed for build compatibility
  
  // State management
  const [activeTab, setActiveTab] = useState<'overview' | 'memberships' | 'proposals' | 'voting' | 'delegations'>('overview');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // Data state
  const [daoMemberships, setDAOMemberships] = useState<DAOMembership[]>([]);
  const [proposals, setProposals] = useState<GovernanceProposal[]>([]);
  const [analytics, setAnalytics] = useState<DAOAnalytics | null>(null);
  const [votingHistory, setVotingHistory] = useState<VotingHistory[]>([]);
  const [delegations, setDelegations] = useState<DelegationRecord[]>([]);
  
  // Filter state
  const [selectedDAO, setSelectedDAO] = useState<string>('all');
  const [selectedNetwork, setSelectedNetwork] = useState<string>('all');
  const [proposalStatusFilter, setProposalStatusFilter] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    if (canManageDAO() || canViewAnalytics()) {
      loadDAOData();
    } else {
      setError('Insufficient permissions to access DAO governance features');
      setLoading(false);
    }
  }, []);

  const loadDAOData = async () => {
    try {
      setLoading(true);
      setError(null);

      // Mock API calls - Replace with actual DAO API endpoints
      const [membershipsResponse, proposalsResponse, analyticsResponse, votingResponse, delegationsResponse] = await Promise.all([
        fetch('/api/admin/dao/memberships', { credentials: 'include' }),
        fetch('/api/admin/dao/proposals', { credentials: 'include' }),
        fetch('/api/admin/dao/analytics', { credentials: 'include' }),
        fetch('/api/admin/dao/voting-history', { credentials: 'include' }),
        fetch('/api/admin/dao/delegations', { credentials: 'include' }),
      ]);

      // For now, use mock data since backend endpoints are not implemented yet
      setDAOMemberships(mockDAOMemberships);
      setProposals(mockProposals);
      setAnalytics(mockAnalytics);
      setVotingHistory(mockVotingHistory);
      setDelegations(mockDelegations);

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load DAO data');
    } finally {
      setLoading(false);
    }
  };

  const handleRefreshProposals = async (daoName: string) => {
    try {
      // Mock API call to refresh proposals for a specific DAO
      const response = await fetch(`/api/admin/dao/proposals/refresh/${daoName}`, {
        method: 'POST',
        credentials: 'include',
      });

      if (response.ok) {
        await loadDAOData(); // Refresh all data
      } else {
        setError('Failed to refresh proposals');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to refresh proposals');
    }
  };

  const filteredMemberships = daoMemberships.filter(membership => {
    const matchesDAO = selectedDAO === 'all' || membership.dao_name === selectedDAO;
    const matchesNetwork = selectedNetwork === 'all' || membership.network === selectedNetwork;
    const matchesSearch = searchTerm === '' || 
      membership.wallet_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
      membership.dao_name.toLowerCase().includes(searchTerm.toLowerCase());
    
    return matchesDAO && matchesNetwork && matchesSearch;
  });

  const filteredProposals = proposals.filter(proposal => {
    const matchesDAO = selectedDAO === 'all' || proposal.dao_name === selectedDAO;
    const matchesNetwork = selectedNetwork === 'all' || proposal.network === selectedNetwork;
    const matchesStatus = proposalStatusFilter === 'all' || proposal.status === proposalStatusFilter;
    const matchesSearch = searchTerm === '' || 
      proposal.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
      proposal.dao_name.toLowerCase().includes(searchTerm.toLowerCase());
    
    return matchesDAO && matchesNetwork && matchesStatus && matchesSearch;
  });

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat('en-US', {
      notation: 'compact',
      maximumFractionDigits: 1,
    }).format(num);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active': return 'text-blue-700 bg-blue-100';
      case 'succeeded': return 'text-green-700 bg-green-100';
      case 'defeated': return 'text-red-700 bg-red-100';
      case 'expired': return 'text-gray-700 bg-gray-100';
      case 'executed': return 'text-purple-700 bg-purple-100';
      default: return 'text-gray-700 bg-gray-100';
    }
  };

  const getMembershipTypeColor = (type: string) => {
    switch (type) {
      case 'token_holder': return 'text-green-700 bg-green-100';
      case 'nft_holder': return 'text-purple-700 bg-purple-100';
      case 'delegated': return 'text-blue-700 bg-blue-100';
      case 'multisig': return 'text-orange-700 bg-orange-100';
      default: return 'text-gray-700 bg-gray-100';
    }
  };

  const getVoteChoiceColor = (choice: string) => {
    switch (choice) {
      case 'for': return 'text-green-700 bg-green-100';
      case 'against': return 'text-red-700 bg-red-100';
      case 'abstain': return 'text-yellow-700 bg-yellow-100';
      default: return 'text-gray-700 bg-gray-100';
    }
  };

  if (!canManageDAO() && !canViewAnalytics()) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6">
            <div className="text-red-800 font-medium">Access Denied</div>
            <div className="text-red-600 mt-2">
              You don't have permission to access DAO governance features.
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-white rounded-lg shadow p-8 text-center">
            <div className="text-gray-600">Loading DAO governance interface...</div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6">
            <div className="text-red-800 font-medium">Error Loading DAO Data</div>
            <div className="text-red-600 mt-2">{error}</div>
            <button
              onClick={() => {
                setError(null);
                loadDAOData();
              }}
              className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    );
  }

  const uniqueDAOs = Array.from(new Set(daoMemberships.map(m => m.dao_name)));
  const uniqueNetworks = Array.from(new Set(daoMemberships.map(m => m.network)));

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">DAO Governance Interface</h1>
          <p className="text-gray-600 mt-2">
            Monitor and manage DAO memberships, proposals, and governance activity
          </p>
          {user && (
            <div className="mt-4 inline-flex items-center space-x-4">
              <span className="text-sm text-gray-500">
                Admin: {user.wallet_address?.slice(0, 8)}...{user.wallet_address?.slice(-4)}
              </span>
              <span className="text-sm text-gray-500">
                DAO Permissions: {user.permissions?.filter(p => p.includes('dao')).length || 0} active
              </span>
            </div>
          )}
        </div>

        {/* Filters */}
        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
            <input
              type="text"
              placeholder="Search addresses, DAOs..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <select
              value={selectedDAO}
              onChange={(e) => setSelectedDAO(e.target.value)}
              className="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All DAOs</option>
              {uniqueDAOs.map(dao => (
                <option key={dao} value={dao}>{dao}</option>
              ))}
            </select>
            <select
              value={selectedNetwork}
              onChange={(e) => setSelectedNetwork(e.target.value)}
              className="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All Networks</option>
              {uniqueNetworks.map(network => (
                <option key={network} value={network}>{network}</option>
              ))}
            </select>
            <select
              value={proposalStatusFilter}
              onChange={(e) => setProposalStatusFilter(e.target.value)}
              className="border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All Statuses</option>
              <option value="active">Active</option>
              <option value="succeeded">Succeeded</option>
              <option value="defeated">Defeated</option>
              <option value="executed">Executed</option>
              <option value="expired">Expired</option>
            </select>
            <button
              onClick={() => {
                setSearchTerm('');
                setSelectedDAO('all');
                setSelectedNetwork('all');
                setProposalStatusFilter('all');
              }}
              className="bg-gray-100 text-gray-700 px-4 py-2 rounded-md text-sm hover:bg-gray-200"
            >
              Clear Filters
            </button>
          </div>
        </div>

        {/* Navigation Tabs */}
        <div className="bg-white rounded-lg shadow mb-6">
          <div className="border-b border-gray-200">
            <nav className="flex space-x-8" aria-label="Tabs">
              {[
                { key: 'overview', label: 'Overview' },
                { key: 'memberships', label: 'DAO Memberships' },
                { key: 'proposals', label: 'Governance Proposals' },
                { key: 'voting', label: 'Voting History' },
                { key: 'delegations', label: 'Delegations' },
              ].map((tab) => (
                <button
                  key={tab.key}
                  onClick={() => setActiveTab(tab.key as any)}
                  className={`py-4 px-1 border-b-2 font-medium text-sm ${
                    activeTab === tab.key
                      ? 'border-blue-500 text-blue-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  {tab.label}
                </button>
              ))}
            </nav>
          </div>
        </div>

        {/* Tab Content */}
        <div className="space-y-6">
          {/* Overview Tab */}
          {activeTab === 'overview' && analytics && (
            <div className="space-y-6">
              {/* Key Metrics */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-6">
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">DAOs Tracked</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analytics.total_daos_tracked}
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Active Members</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {formatNumber(analytics.total_active_members)}
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Governance Tokens</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {formatNumber(analytics.total_governance_tokens)}
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Active Proposals</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analytics.active_proposals}
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Votes (24h)</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analytics.completed_votes_24h}
                  </div>
                </div>
              </div>

              {/* Top DAOs */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Top DAOs by Members</h3>
                <div className="space-y-4">
                  {analytics.top_daos_by_members.map((dao, index) => (
                    <div key={dao.dao_name} className="flex items-center justify-between">
                      <div className="flex items-center space-x-3">
                        <div className="flex-shrink-0 w-6 h-6 bg-blue-100 rounded-full flex items-center justify-center text-xs font-medium text-blue-600">
                          {index + 1}
                        </div>
                        <span className="text-sm font-medium text-gray-900">{dao.dao_name}</span>
                      </div>
                      <div className="text-right">
                        <div className="text-sm font-medium text-gray-900">{dao.members} members</div>
                        <div className="text-xs text-gray-500">{formatNumber(dao.total_voting_power)} voting power</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Network Distribution */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Network Distribution</h3>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  {Object.entries(analytics.network_distribution).map(([network, count]) => (
                    <div key={network} className="text-center">
                      <div className="text-2xl font-bold text-gray-900">{count}</div>
                      <div className="text-sm text-gray-600 capitalize">{network}</div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Memberships Tab */}
          {activeTab === 'memberships' && (
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200 flex justify-between items-center">
                <h3 className="text-lg font-medium text-gray-900">
                  DAO Memberships ({filteredMemberships.length})
                </h3>
                <button
                  onClick={() => loadDAOData()}
                  className="bg-blue-600 text-white px-4 py-2 rounded text-sm hover:bg-blue-700"
                >
                  Refresh Data
                </button>
              </div>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Member
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        DAO
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Type
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Voting Power
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Network
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Last Vote
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Delegation
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {filteredMemberships.map((membership) => (
                      <tr key={membership.id} className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 font-mono">
                          {membership.wallet_address.slice(0, 8)}...{membership.wallet_address.slice(-4)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {membership.dao_name}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getMembershipTypeColor(membership.membership_type)}`}>
                            {membership.membership_type.replace('_', ' ')}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatNumber(membership.voting_power)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 capitalize">
                          {membership.network}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {membership.last_vote ? formatDate(membership.last_vote) : 'Never'}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {membership.delegation_status === 'none' ? 'Self' :
                           membership.delegation_status === 'delegated_to' ? 'Delegated' :
                           'Delegate'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {filteredMemberships.length === 0 && (
                  <div className="text-center py-8 text-gray-500">
                    No DAO memberships found matching the current filters
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Proposals Tab */}
          {activeTab === 'proposals' && (
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">
                  Governance Proposals ({filteredProposals.length})
                </h3>
              </div>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Proposal
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        DAO
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Status
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Votes
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Deadline
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {filteredProposals.map((proposal) => (
                      <tr key={proposal.id} className="hover:bg-gray-50">
                        <td className="px-6 py-4 text-sm text-gray-900">
                          <div className="font-medium">{proposal.title}</div>
                          <div className="text-gray-500 text-xs">#{proposal.proposal_id}</div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {proposal.dao_name}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(proposal.status)}`}>
                            {proposal.status}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <div className="space-y-1">
                            <div className="text-green-600">For: {formatNumber(proposal.votes_for)}</div>
                            <div className="text-red-600">Against: {formatNumber(proposal.votes_against)}</div>
                            {proposal.votes_abstain > 0 && (
                              <div className="text-yellow-600">Abstain: {formatNumber(proposal.votes_abstain)}</div>
                            )}
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatDate(proposal.voting_deadline)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                          <a
                            href={proposal.proposal_url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-600 hover:text-blue-900 mr-3"
                          >
                            View
                          </a>
                          <button
                            onClick={() => handleRefreshProposals(proposal.dao_name)}
                            className="text-gray-600 hover:text-gray-900"
                          >
                            Refresh
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {filteredProposals.length === 0 && (
                  <div className="text-center py-8 text-gray-500">
                    No proposals found matching the current filters
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Voting History Tab */}
          {activeTab === 'voting' && (
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">Recent Voting Activity</h3>
              </div>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Voter
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Proposal
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Vote
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Voting Power
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Date
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Transaction
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {votingHistory.map((vote) => (
                      <tr key={vote.id} className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 font-mono">
                          {vote.wallet_address.slice(0, 8)}...{vote.wallet_address.slice(-4)}
                        </td>
                        <td className="px-6 py-4 text-sm text-gray-900">
                          <div className="font-medium">{vote.proposal_title}</div>
                          <div className="text-gray-500 text-xs">{vote.dao_name}</div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium capitalize ${getVoteChoiceColor(vote.vote_choice)}`}>
                            {vote.vote_choice}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatNumber(vote.voting_power_used)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatDate(vote.voted_at)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <a
                            href={`https://etherscan.io/tx/${vote.transaction_hash}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-600 hover:text-blue-900 font-mono"
                          >
                            {vote.transaction_hash.slice(0, 8)}...{vote.transaction_hash.slice(-4)}
                          </a>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {votingHistory.length === 0 && (
                  <div className="text-center py-8 text-gray-500">
                    No voting history found
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Delegations Tab */}
          {activeTab === 'delegations' && (
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">Delegation Records</h3>
              </div>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Delegator
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Delegatee
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        DAO
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Voting Power
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Status
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Date
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {delegations.map((delegation) => (
                      <tr key={delegation.id} className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 font-mono">
                          {delegation.delegator.slice(0, 8)}...{delegation.delegator.slice(-4)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 font-mono">
                          {delegation.delegatee.slice(0, 8)}...{delegation.delegatee.slice(-4)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {delegation.dao_name}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatNumber(delegation.voting_power)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                            delegation.is_active ? 'text-green-700 bg-green-100' : 'text-gray-700 bg-gray-100'
                          }`}>
                            {delegation.is_active ? 'Active' : 'Inactive'}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatDate(delegation.delegated_at)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {delegations.length === 0 && (
                  <div className="text-center py-8 text-gray-500">
                    No delegation records found
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// Mock data for development (replace with actual API calls)
const mockDAOMemberships: DAOMembership[] = [
  {
    id: '1',
    wallet_address: '0x742d35Cc6DbfC5B3bDd5c8e8E0C7b8eF5d5A2dA1',
    dao_name: 'MakerDAO',
    dao_contract: '0x9f8f72aa9304c8b593d555f12ef6589cc3a579a2',
    network: 'ethereum',
    membership_type: 'token_holder',
    governance_tokens: 1250.5,
    voting_power: 1250.5,
    joined_at: '2023-06-15T00:00:00Z',
    last_vote: '2024-01-18T10:30:00Z',
    delegation_status: 'none',
  },
  {
    id: '2',
    wallet_address: '0x8ba1f109551bD432803012645Hac136c22Fd5B',
    dao_name: 'Compound',
    dao_contract: '0xc00e94cb662c3520282e6f5717214004a7f26888',
    network: 'ethereum',
    membership_type: 'delegated',
    governance_tokens: 0,
    voting_power: 500.0,
    joined_at: '2023-08-20T00:00:00Z',
    last_vote: '2024-01-15T14:20:00Z',
    delegation_status: 'delegated_from',
    delegated_from: ['0x123456789abcdef123456789abcdef123456789a'],
  },
];

const mockProposals: GovernanceProposal[] = [
  {
    id: '1',
    dao_name: 'MakerDAO',
    proposal_id: 'MIP-45',
    title: 'Increase Stability Fee for ETH-A Vault Type',
    description: 'Proposal to increase stability fee to manage DAI supply',
    proposer: '0x742d35Cc6DbfC5B3bDd5c8e8E0C7b8eF5d5A2dA1',
    status: 'active',
    votes_for: 125000,
    votes_against: 45000,
    votes_abstain: 5000,
    total_votes: 175000,
    quorum_required: 100000,
    voting_deadline: '2024-01-25T23:59:59Z',
    network: 'ethereum',
    proposal_url: 'https://vote.makerdao.com/polling/QmMIP45',
  },
  {
    id: '2',
    dao_name: 'Compound',
    proposal_id: '156',
    title: 'Add support for new collateral asset',
    description: 'Proposal to add WSTETH as collateral',
    proposer: '0x8ba1f109551bD432803012645Hac136c22Fd5B',
    status: 'succeeded',
    votes_for: 680000,
    votes_against: 120000,
    votes_abstain: 25000,
    total_votes: 825000,
    quorum_required: 400000,
    voting_deadline: '2024-01-20T23:59:59Z',
    execution_deadline: '2024-01-27T23:59:59Z',
    network: 'ethereum',
    proposal_url: 'https://compound.finance/governance/proposals/156',
  },
];

const mockAnalytics: DAOAnalytics = {
  total_daos_tracked: 24,
  total_active_members: 15420,
  total_governance_tokens: 2450000,
  active_proposals: 12,
  completed_votes_24h: 145,
  top_daos_by_members: [
    { dao_name: 'MakerDAO', members: 3250, total_voting_power: 650000 },
    { dao_name: 'Compound', members: 2890, total_voting_power: 580000 },
    { dao_name: 'Aave', members: 2650, total_voting_power: 520000 },
    { dao_name: 'Uniswap', members: 2100, total_voting_power: 420000 },
  ],
  governance_activity: [],
  network_distribution: {
    ethereum: 18,
    polygon: 4,
    arbitrum: 2,
  },
};

const mockVotingHistory: VotingHistory[] = [
  {
    id: '1',
    wallet_address: '0x742d35Cc6DbfC5B3bDd5c8e8E0C7b8eF5d5A2dA1',
    dao_name: 'MakerDAO',
    proposal_id: 'MIP-45',
    proposal_title: 'Increase Stability Fee for ETH-A Vault Type',
    vote_choice: 'for',
    voting_power_used: 1250.5,
    voted_at: '2024-01-18T10:30:00Z',
    transaction_hash: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    network: 'ethereum',
  },
  {
    id: '2',
    wallet_address: '0x8ba1f109551bD432803012645Hac136c22Fd5B',
    dao_name: 'Compound',
    proposal_id: '156',
    proposal_title: 'Add support for new collateral asset',
    vote_choice: 'for',
    voting_power_used: 500.0,
    voted_at: '2024-01-15T14:20:00Z',
    transaction_hash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    network: 'ethereum',
  },
];

const mockDelegations: DelegationRecord[] = [
  {
    id: '1',
    delegator: '0x123456789abcdef123456789abcdef123456789a',
    delegatee: '0x8ba1f109551bD432803012645Hac136c22Fd5B',
    dao_name: 'Compound',
    voting_power: 500.0,
    delegated_at: '2023-12-01T00:00:00Z',
    is_active: true,
    network: 'ethereum',
    transaction_hash: '0xdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abc',
  },
];