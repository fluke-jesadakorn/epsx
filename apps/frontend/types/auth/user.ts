export interface User {
  id: string;
  email: string;
  createdAt: string;
  updatedAt: string;
  emailVerified?: boolean;
  role: 'USER' | 'ADMIN';
  displayName?: string;
  photoURL?: string;
}

export interface UserCredentials {
  email: string;
  password: string;
}
