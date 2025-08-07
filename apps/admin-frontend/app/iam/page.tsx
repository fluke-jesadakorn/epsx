import { IAMDashboard } from '@/components/admin/IAMDashboard';
import { getIAMUsers, getIAMRoles, getIAMPolicies } from '@epsx/server-actions';

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
