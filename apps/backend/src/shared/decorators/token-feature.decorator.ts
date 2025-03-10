import { SetMetadata, UseGuards, applyDecorators } from "@nestjs/common";
import { TokenFeature } from "../types/roles.enum";
import { FirebaseAuthGuard } from "../guards/firebase-auth.guard";
import { TokenFeatureGuard } from "../guards/token-feature.guard";

export const TOKEN_FEATURE_KEY = "token_feature";

export const RequiresTokenFeature = (...features: TokenFeature[]) => {
  return applyDecorators(
    SetMetadata(TOKEN_FEATURE_KEY, features),
    UseGuards(FirebaseAuthGuard, TokenFeatureGuard)
  );
};

// Combine role and token feature requirements
export const RequiresFeature = (feature: TokenFeature) => {
  return applyDecorators(RequiresTokenFeature(feature));
};
