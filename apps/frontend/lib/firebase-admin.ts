import { initializeApp, getApps, cert } from 'firebase-admin/app';
import { getAuth } from 'firebase-admin/auth';
import serviceAccount from '../../backend/secret/firebase-service-account.json';

const app = getApps().length 
  ? getApps()[0] 
  : initializeApp({
      credential: cert(serviceAccount as any)
    });

const auth = getAuth(app);

export { auth, app };
