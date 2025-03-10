import { Request } from 'express';

export interface FirebaseUserData {
  uid: string;
  email: string | undefined;
  customClaims: { [key: string]: any } | undefined;
}

export interface AuthenticatedRequest extends Request {
  user: FirebaseUserData;
}
