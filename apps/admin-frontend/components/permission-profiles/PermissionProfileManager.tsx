/**
 * Permission Profile Manager - Main CRUD Interface for Permission Profiles
 */

'use client'

import { useState, useEffect } from 'react'
import { Plus, Search, Filter, Settings, Shield } from 'lucide-react'
import { Button } from '@epsx/ui'
import { Input } from '@epsx/ui'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@epsx/ui'
import { Badge } from '@epsx/ui'
import { PermissionProfileCard } from './PermissionProfileCard'
import { PermissionProfileModal } from './PermissionProfileModal'
import { usePermissionProfiles } from '@/hooks/use-permission-profiles'
import type { PermissionProfile } from '@/lib/types/permission-profiles'

export function PermissionProfileManager() {
  const [searchTerm, setSearchTerm] = useState('')
  const [categoryFilter, setCategoryFilter] = useState<string>('all')
  const [tierFilter, setTierFilter] = useState<string>('all')
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [editingProfile, setEditingProfile] = useState<PermissionProfile | null>(null)

  const { 
    profiles, 
    loading, 
    error,
    pagination,
    refreshProfiles,
    createProfile,
    updateProfile,
    deleteProfile
  } = usePermissionProfiles({
    search: searchTerm,
    category: categoryFilter === 'all' ? undefined : categoryFilter,
    activeOnly: true
  })

  const handleCreateProfile = async (profileData: Partial<PermissionProfile>) => {
    const result = await createProfile(profileData)
    if (result.success) {
      setShowCreateModal(false)
      refreshProfiles()
    }
    return result
  }

  const handleUpdateProfile = async (id: string, profileData: Partial<PermissionProfile>) => {
    const result = await updateProfile(id, profileData)
    if (result.success) {
      setEditingProfile(null)
      refreshProfiles()
    }
    return result
  }

  const handleDeleteProfile = async (id: string) => {
    if (confirm('Are you sure you want to delete this permission profile? This action cannot be undone.')) {
      const result = await deleteProfile(id)
      if (result.success) {
        refreshProfiles()
      }
    }
  }

  const filteredProfiles = profiles.filter(profile => {
    const matchesSearch = !searchTerm || 
      profile.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      profile.description.toLowerCase().includes(searchTerm.toLowerCase())
    
    const matchesCategory = categoryFilter === 'all' || profile.category === categoryFilter
    const matchesTier = tierFilter === 'all' || profile.targetTier === tierFilter
    
    return matchesSearch && matchesCategory && matchesTier
  })

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Shield className="h-6 w-6 text-purple-500" />
            Permission Profiles
          </h1>
          <p className="text-muted-foreground mt-1">
            Manage permission profiles and role templates
          </p>
        </div>
        <Button onClick={() => setShowCreateModal(true)} className="gap-2">
          <Plus className="h-4 w-4" />
          Create Profile
        </Button>
      </div>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1 relative">
          <Search className="h-4 w-4 absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search profiles by name or description..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-10"
          />
        </div>
        
        <Select value={categoryFilter} onValueChange={setCategoryFilter}>
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Category" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Categories</SelectItem>
            <SelectItem value="user">User</SelectItem>
            <SelectItem value="moderator">Moderator</SelectItem>
            <SelectItem value="admin">Admin</SelectItem>
            <SelectItem value="custom">Custom</SelectItem>
            <SelectItem value="business">Business</SelectItem>
            <SelectItem value="technical">Technical</SelectItem>
          </SelectContent>
        </Select>

        <Select value={tierFilter} onValueChange={setTierFilter}>
          <SelectTrigger className="w-[150px]">
            <SelectValue placeholder="Tier" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Tiers</SelectItem>
            <SelectItem value="free">Free</SelectItem>
            <SelectItem value="bronze">Bronze</SelectItem>
            <SelectItem value="silver">Silver</SelectItem>
            <SelectItem value="gold">Gold</SelectItem>
            <SelectItem value="platinum">Platinum</SelectItem>
            <SelectItem value="admin">Admin</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="pancake-card p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">{filteredProfiles.length}</p>
              <p className="text-sm text-muted-foreground">Total Profiles</p>
            </div>
            <Shield className="h-8 w-8 text-blue-500" />
          </div>
        </div>
        
        <div className="pancake-card p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">
                {filteredProfiles.filter(p => p.category === 'user').length}
              </p>
              <p className="text-sm text-muted-foreground">User Profiles</p>
            </div>
            <Badge variant="secondary" className="h-8">User</Badge>
          </div>
        </div>

        <div className="pancake-card p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">
                {filteredProfiles.filter(p => p.category === 'admin').length}
              </p>
              <p className="text-sm text-muted-foreground">Admin Profiles</p>
            </div>
            <Badge variant="destructive" className="h-8">Admin</Badge>
          </div>
        </div>

        <div className="pancake-card p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">
                {filteredProfiles.filter(p => p.category === 'custom').length}
              </p>
              <p className="text-sm text-muted-foreground">Custom Profiles</p>
            </div>
            <Badge variant="outline" className="h-8">Custom</Badge>
          </div>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      )}

      {/* Loading State */}
      {loading && (
        <div className="flex items-center justify-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600"></div>
        </div>
      )}

      {/* Profiles Grid */}
      {!loading && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredProfiles.map((profile) => (
            <PermissionProfileCard
              key={profile.id}
              profile={profile}
              onEdit={() => setEditingProfile(profile)}
              onDelete={() => handleDeleteProfile(profile.id)}
            />
          ))}
        </div>
      )}

      {/* Empty State */}
      {!loading && filteredProfiles.length === 0 && (
        <div className="text-center py-12">
          <Shield className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">No permission profiles found</h3>
          <p className="text-muted-foreground mb-4">
            {searchTerm || categoryFilter !== 'all' || tierFilter !== 'all'
              ? 'Try adjusting your search filters.'
              : 'Get started by creating your first permission profile.'
            }
          </p>
          {(!searchTerm && categoryFilter === 'all' && tierFilter === 'all') && (
            <Button onClick={() => setShowCreateModal(true)} className="gap-2">
              <Plus className="h-4 w-4" />
              Create First Profile
            </Button>
          )}
        </div>
      )}

      {/* Create Modal */}
      <PermissionProfileModal
        open={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onSubmit={handleCreateProfile}
        title="Create Permission Profile"
        submitText="Create Profile"
      />

      {/* Edit Modal */}
      <PermissionProfileModal
        open={!!editingProfile}
        onClose={() => setEditingProfile(null)}
        onSubmit={(data) => handleUpdateProfile(editingProfile!.id, data)}
        initialData={editingProfile || undefined}
        title="Edit Permission Profile"
        submitText="Update Profile"
      />
    </div>
  )
}