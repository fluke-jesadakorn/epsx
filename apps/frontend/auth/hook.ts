import { useAuth } from './ctx';

export const useAuthData = () => {
  const { usr, loading } = useAuth();
  
  return {
    usr,
    loading,
    isAuth: !!usr,
    hasRole: (role: string) => usr?.roles.includes(role) ?? false,
    isAdmin: usr?.roles.includes('admin') ?? false,
  };
};

export const useAuthActions = () => {
  const { login, logout } = useAuth();
  
  return {
    login,
    logout,
  };
};