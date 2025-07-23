import { useAdminAuth } from './ctx';

export const useAdminAuthData = () => {
  const { user, loading } = useAdminAuth();
  
  return {
    user,
    loading,
    isAuth: !!user,
    isAdmin: user?.isAdmin ?? false,
    hasRole: (role: string) => user?.roles.includes(role) ?? false,
  };
};

export const useAdminAuthActions = () => {
  const { login, logout } = useAdminAuth();
  
  return {
    login,
    logout,
  };
};