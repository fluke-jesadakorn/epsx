import { NextRequest } from 'next/server';
import { createOAuthInitiation, createFrontendOAuthConfig } from '../../../../../../shared/auth/oauth-initiate';
import { getFrontendUrl } from '../../../../../../shared/utils/url-resolver';

export async function POST(request: NextRequest) {
  const { redirectTo } = await request.json();
  
  const frontendUrl = getFrontendUrl('server');
  const redirectUri = `${frontendUrl}/api/auth/callback/epsx-backend`;
  
  const config = createFrontendOAuthConfig(redirectUri);
  
  return createOAuthInitiation(config, { redirectTo });
}