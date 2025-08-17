import { IAMDashboard } from '@/components/admin/IAMDashboard';

// TODO: Replace with direct API calls
// import { getIAMUsers, getIAMRoles, getIAMPolicies } from '@epsx/server-actions';

// Temporary placeholder functions for migration
const getIAMUsers = async () => [];
const getIAMRoles = async () => [];
const getIAMPolicies = async () => [];

export default async function IAMPage() {
  // Fetch IAM data server-side
  const [usersResult, rolesResult, policiesResult] = await Promise.allSettled([
    getIAMUsers(),
    getIAMRoles(),
    getIAMPolicies()
  ]);

  const users = usersResult.status === 'fulfilled' ? usersResult.value : [];
  const roles = rolesResult.status === 'fulfilled' ? rolesResult.value : [];
  const policies = policiesResult.status === 'fulfilled' ? policiesResult.value : [];

  return (
    <IAMDashboard 
      initialUsers={users}
      initialRoles={roles}
      initialPolicies={policies}
    />
  );
}
