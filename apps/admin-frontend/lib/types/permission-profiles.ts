/**
 * Permission Profile Types
 */

export interface Permission {
  resource: string
  action: string
}

export interface PermissionProfile {
  id: string
  name: string
  description: string
  category: string
  permissions: Permission[]
  targetTier: string
  isActive: boolean
  createdAt: string
  updatedAt: string
  createdBy: string
  assignmentCount?: number
}

export interface PermissionProfileQuery {
  page?: number
  limit?: number
  category?: string
  activeOnly?: boolean
  name?: string
}

export interface ListPermissionProfilesResponse {
  profiles: PermissionProfile[]
  totalCount: number
  page: number
  limit: number
  hasMore: boolean
}

export interface ValidateAssignmentRequest {
  userId: string
  profileId: string
}

export interface ValidateAssignmentResponse {
  isValid: boolean
  errors: string[]
  warnings: string[]
  conflicts: string[]
  recommendations: string[]
}

export interface BulkValidateAssignmentRequest {
  userIds: string[]
  profileId: string
}

export interface BulkValidationResult {
  userId: string
  isValid: boolean
  errors: string[]
  warnings: string[]
}

export interface BulkValidateAssignmentResponse {
  profileId: string
  profileName: string
  totalUsers: number
  validAssignments: number
  invalidAssignments: number
  results: BulkValidationResult[]
}

export interface UnassignProfileRequest {
  userId: string
  profileId: string
  reason?: string
}

export interface UnassignProfileResponse {
  success: boolean
  message: string
  userId: string
  profileId: string
  unassignedAt: string
}

export interface PermissionCategory {
  id: string
  name: string
  description: string
}

export interface PermissionTier {
  id: string
  name: string
  description: string
}

export interface PermissionProfileFormData {
  name: string
  description: string
  category: string
  targetTier: string
  permissions: Permission[]
  isActive: boolean
}

// API Response wrapper
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: {
    message: string
    code?: string
  }
}