import {
  Controller,
  Get,
  Post,
  Delete,
  Body,
  Req,
  Res,
  Param,
  Query,
  UnauthorizedException,
  ForbiddenException,
  UseGuards,
} from "@nestjs/common";
import { Response } from "express";
import { AuthenticatedRequest } from "./interfaces/auth-request.interface";
import { FirebaseAdminService } from "../../shared/firebase-admin";
import { Roles } from "../../shared/decorators/roles.decorator";
import { UserRole } from "../../shared/types/roles.enum";
import { AuthService } from "./services/auth.service";
import { TokenService } from "./services/token.service";
import { SessionService } from "./services/session.service";
import { AuthLoggerService } from "./services/auth-logger.service";
import { UserManagementService } from "./services/user-management.service";
import { FirebaseAuthGuard } from "../../shared/guards/role.guard";
import { 
  ApiTags, 
  ApiOperation, 
  ApiResponse, 
  ApiCookieAuth,
  ApiParam,
  ApiBody,
  ApiBearerAuth,
  ApiQuery,
  ApiSecurity
} from '@nestjs/swagger';

@ApiTags('Authentication')
@Controller("auth")
@UseGuards(FirebaseAuthGuard)
export class AuthController {
  constructor(
    private readonly firebaseAdmin: FirebaseAdminService,
    private readonly authService: AuthService,
    private readonly tokenService: TokenService,
    private readonly sessionService: SessionService,
    private readonly authLogger: AuthLoggerService,
    private readonly userManagementService: UserManagementService
  ) {}

  @Post("oauth")
  @ApiOperation({
    summary: 'Initialize OAuth flow',
    description: 'Starts the OAuth authentication process for the specified provider'
  })
  @ApiBody({
    schema: {
      type: 'object',
      required: ['provider'],
      properties: {
        provider: {
          type: 'string',
          example: 'google',
          description: 'OAuth provider (e.g. google)'
        }
      }
    }
  })
  async initializeOAuth(
    @Body() { provider }: { provider: string },
    @Req() req: AuthenticatedRequest
  ) {
    if (!provider) {
      throw new UnauthorizedException("Provider is required");
    }
    const { authUrl } = await this.authService.generateOAuthURL(provider);
    await this.authLogger.logAuthEvent({
      action: 'login',
      status: 'success',
      metadata: { provider, initOAuth: true },
      ipAddress: req.ip || 'unknown',
      userAgent: req.headers['user-agent'] || 'unknown'
    });
    return { authUrl };
  }

  @Get("oauth/callback")
  @ApiOperation({
    summary: 'Handle OAuth callback',
    description: 'Processes the OAuth callback from authentication provider'
  })
  @ApiQuery({
    name: 'code',
    required: true,
    type: String,
    description: 'Authorization code from OAuth provider'
  })
  @ApiQuery({
    name: 'state',
    required: true,
    type: String,
    description: 'State parameter for CSRF protection'
  })
  async handleOAuthCallback(
    @Req() req: AuthenticatedRequest,
    @Res() res: Response,
    @Query('code') code: string,
    @Query('state') state: string
  ) {
    const redirectUrl = await this.authService.handleOAuthCallbackAndCreateSession(
      code,
      state,
      req.ip || 'unknown',
      req.headers['user-agent'] || 'unknown',
      res
    );
    return res.redirect(redirectUrl);
  }

  @ApiOperation({
    summary: 'Verify session',
    description: 'Verifies the session and returns user information'
  })
  @ApiCookieAuth()
  @Get("verify")
  async verify(@Req() req: AuthenticatedRequest) {
    return this.authService.verifySession(req.cookies?.__session);
  }

  @ApiOperation({
    summary: 'Get user sessions',
    description: 'Get all active sessions for a user'
  })
  @ApiParam({
    name: 'userId',
    description: 'User ID to get sessions for',
    required: true
  })
  @ApiBearerAuth('JWT-auth')
  @Get("sessions/:userId")
  @Roles(UserRole.ADMINISTRATOR)
  async getUserSessions(@Param('userId') userId: string) {
    return this.sessionService.getActiveSessions(userId);
  }

  @ApiOperation({
    summary: 'Invalidate session',
    description: 'Invalidate a specific session'
  })
  @ApiParam({
    name: 'sessionId',
    description: 'Session ID to invalidate',
    required: true
  })
  @ApiBearerAuth('JWT-auth')
  @Delete("sessions/:sessionId")
  @Roles(UserRole.ADMINISTRATOR)
  async invalidateSession(
    @Param('sessionId') sessionId: string,
    @Body() request: { reason: string },
    @Req() req: AuthenticatedRequest
  ) {
    const adminId = req.user?.uid;
    if (!adminId) {
      throw new ForbiddenException('Admin ID not found');
    }

    await this.sessionService.invalidateSession(sessionId, {
      reason: request.reason,
      adminId
    });

    return { success: true };
  }

  @ApiOperation({
    summary: 'Invalidate all user sessions',
    description: 'Invalidate all sessions for a specific user'
  })
  @ApiParam({
    name: 'userId',
    description: 'User ID to invalidate sessions for',
    required: true
  })
  @ApiBearerAuth('JWT-auth')
  @Delete("sessions/user/:userId")
  @Roles(UserRole.ADMINISTRATOR)
  async invalidateUserSessions(
    @Param('userId') userId: string,
    @Body() request: { reason: string },
    @Req() req: AuthenticatedRequest
  ) {
    const adminId = req.user?.uid;
    if (!adminId) {
      throw new ForbiddenException('Admin ID not found');
    }

    await this.sessionService.invalidateAllUserSessions(userId, {
      reason: request.reason,
      adminId
    });

    return { success: true };
  }

  @ApiOperation({
    summary: 'Get auth logs',
    description: 'Get authentication logs for a user'
  })
  @ApiParam({
    name: 'userId',
    description: 'User ID to get logs for',
    required: true
  })
  @ApiQuery({
    name: 'startDate',
    required: false,
    type: Date
  })
  @ApiQuery({
    name: 'endDate',
    required: false,
    type: Date
  })
  @ApiQuery({
    name: 'onlySuspicious',
    required: false,
    type: Boolean
  })
  @ApiBearerAuth('JWT-auth')
  @Get("logs/:userId")
  @Roles(UserRole.ADMINISTRATOR)
  async getAuthLogs(
    @Param('userId') userId: string,
    @Query('startDate') startDate?: string,
    @Query('endDate') endDate?: string,
    @Query('onlySuspicious') onlySuspicious?: boolean
  ) {
    const start = startDate ? new Date(startDate) : undefined;
    const end = endDate ? new Date(endDate) : undefined;

    return this.authLogger.getAuthLogs(
      userId,
      start,
      end,
      onlySuspicious
    );
  }

  @ApiOperation({
    summary: 'Assign role to user',
    description: 'Assigns a role to a specified user (Admin only)'
  })
  @ApiParam({
    name: 'userId',
    description: 'User ID',
    required: true
  })
  @ApiBearerAuth('JWT-auth')
  @Post("roles/:userId")
  @Roles(UserRole.ADMINISTRATOR)
  async assignRole(
    @Param("userId") userId: string,
    @Body() { role }: { role: UserRole },
    @Req() req: AuthenticatedRequest
  ) {
    const adminId = req.user?.uid;
    if (!adminId) {
      throw new ForbiddenException('Admin ID not found');
    }

    await this.userManagementService.assignUserRole(userId, role, adminId);
    return { success: true };
  }

  @ApiOperation({
    summary: 'Get all users with roles',
    description: 'Retrieves a list of all users and their roles (Admin only)'
  })
  @ApiBearerAuth('JWT-auth')
  @Get("roles")
  @Roles(UserRole.ADMINISTRATOR)
  async getUsers() {
    return this.userManagementService.getUsers();
  }
}
