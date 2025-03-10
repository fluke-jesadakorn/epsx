import { Injectable, CanActivate, ExecutionContext, ForbiddenException } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { TOKEN_FEATURE_KEY } from '../decorators/token-feature.decorator';
import { TokenFeature } from '../types/roles.enum';
import { TokenClaims } from '../types/token-claims';
import { canAccessFeature } from '../types/features';

@Injectable()
export class TokenFeatureGuard implements CanActivate {
  constructor(private reflector: Reflector) {}

  canActivate(context: ExecutionContext): boolean {
    const requiredFeatures = this.reflector.getAllAndOverride<TokenFeature[]>(
      TOKEN_FEATURE_KEY,
      [context.getHandler(), context.getClass()]
    );

    if (!requiredFeatures?.length) {
      return true;
    }

    const request = context.switchToHttp().getRequest();
    const user = request.user as TokenClaims;

    if (!user) {
      throw new ForbiddenException('User not authenticated');
    }

    // Check if user has access to all required features
    const hasAllFeatures = requiredFeatures.every(feature => 
      canAccessFeature(user.role, user.tokenBalance, feature)
    );

    if (!hasAllFeatures) {
      const requiredTokens = Math.max(...requiredFeatures.map(feature => {
        const requirement = this.getFeatureRequirement(feature);
        return requirement?.minTokens || 0;
      }));

      throw new ForbiddenException(
        `Insufficient token balance. Required: ${requiredTokens}, Current: ${user.tokenBalance}`
      );
    }

    return true;
  }

  private getFeatureRequirement(feature: TokenFeature) {
    // This would be imported from your features configuration
    const { TOKEN_FEATURE_REQUIREMENTS } = require('../types/features');
    return TOKEN_FEATURE_REQUIREMENTS[feature];
  }
}
