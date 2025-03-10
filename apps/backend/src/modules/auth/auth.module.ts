import { Module } from "@nestjs/common";
import { MongooseModule } from "@nestjs/mongoose";
import { AuthController } from "./auth.controller";
import { AuthService } from "./services/auth.service";
import { TokenService } from "./services/token.service";
import { SessionService } from "./services/session.service";
import { AuthLoggerService } from "./services/auth-logger.service";
import { RoleService } from "./services/role.service";
import { UserManagementService } from "./services/user-management.service";
import { SharedModule } from "../../shared/shared.module";
import { AdminGuard } from "./guards/admin.guard";
import { AuthLog, AuthLogSchema } from "@epsx/shared";

// Export permissions and decorators
export { PERMISSIONS, PermissionKey } from "./constants/permissions";
export {
  TokenFeature,
  RequireFeatures,
  RequirePortfolioManagement,
  RequireTradingBot,
  RequireAIAnalysis,
  RequirePortfolioAutomation,
} from "./decorators/feature.decorator";

@Module({
  imports: [
    MongooseModule.forFeature([{ name: AuthLog.name, schema: AuthLogSchema }]),
    SharedModule,
  ],
  controllers: [AuthController],
  providers: [
    AuthService,
    TokenService,
    SessionService,
    AuthLoggerService,
    RoleService,
    UserManagementService,
    AdminGuard,
  ],
  exports: [
    AuthService,
    TokenService,
    SessionService,
    AuthLoggerService,
    RoleService,
    UserManagementService,
    AdminGuard,
  ],
})
export class AuthModule {}
