import { NextRequest } from 'next/server';
import { createOAuthInitiation, createAdminOAuthConfig } from '../../../../../../shared/auth/oauth-initiate';
import { URL, URLContext, Service } from '../../../../../../shared/utils/url-resolver';

export async function POST(request: NextRequest) {
  const { redirectTo } = await request.json();
  
  const adminUrl = URL.get(Service.ADMIN, URLContext.SERVER);
  const redirectUri = `${adminUrl}/api/auth/callback/epsx-backend`;
  
  const config = createAdminOAuthConfig(redirectUri);
  
  return createOAuthInitiation(config, { redirectTo });
}