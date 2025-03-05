import {
  Controller,
  Post,
  Get,
  Body,
  Param,
  Query,
  BadRequestException,
  UnauthorizedException,
  Headers,
  Res,
  Req
} from '@nestjs/common';
import { AuthService } from './auth.service';
import { UserRole } from '../../shared/guards/role.guard';
import { OAuthInitDTO } from './dto/oauth.dto';
import { GoogleSignInDTO } from './dto/google-signin.dto';
import { FirebaseAdminService } from '../../shared/firebase-admin';
import { Response, Request } from 'express';

@Controller('auth')
export class AuthController {
  constructor(
    private readonly authService: AuthService,
    private readonly firebaseAdmin: FirebaseAdminService
  ) {}

  @Post('google')
  async signInWithGoogle(
    @Body() googleSignInDto: GoogleSignInDTO,
    @Res() res: Response
  ) {
    const { token, user } = await this.authService.signInWithGoogle(googleSignInDto.idToken);
    const userRole = await this.authService.getUserRole(user.uid);

    // Set cookies with proper types for express
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/',
      maxAge: 7 * 24 * 60 * 60 * 1000 // 1 week
    };

    res.cookie('__session', token, cookieOptions);
    res.cookie('uid', user.uid, cookieOptions);
    res.cookie('email', user.email || '', cookieOptions);
    res.cookie('role', userRole.role, cookieOptions);

    return res.json({
      user: {
        uid: user.uid,
        email: user.email,
        displayName: user.displayName,
        photoURL: user.photoURL,
        role: userRole.role
      },
      redirectUrl: googleSignInDto.redirectUrl || '/home'
    });
  }

  @Post('oauth')
  async initializeOAuth(@Body() oauthInitDto: OAuthInitDTO) {
    return await this.authService.generateOAuthURL(oauthInitDto.provider);
  }

  @Get('/oauth/callback')
  async handleOAuthCallback(
    @Res() res: Response,
    @Query('code') code: string,
    @Query('state') state: string,
    @Query('error') error?: string
  ) {
    console.log('OAuth Callback Received:', { code: code?.substring(0, 10) + '...', state, error });
    if (error) {
      const frontendErrorUrl = new URL('/login', process.env.FRONTEND_URL!);
      frontendErrorUrl.searchParams.set('error', error);
      return res.redirect(frontendErrorUrl.toString());
    }

    try {
      console.log('Processing OAuth callback...');
      const { token, user } = await this.authService.handleOAuthCallback(code, state);
      console.log('OAuth callback processed successfully');

      // Get user role
      const userRole = await this.authService.getUserRole(user.uid);
      
      // Set cookies with proper types for express
      const cookieOptions = {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax' as const,
        path: '/',
        maxAge: 7 * 24 * 60 * 60 * 1000 // 1 week
      };

      res.cookie('__session', token, cookieOptions);
      res.cookie('uid', user.uid, cookieOptions);
      res.cookie('email', user.email || '', cookieOptions);
      res.cookie('role', userRole.role, cookieOptions);

      // Redirect to frontend home page
      return res.redirect(`${process.env.FRONTEND_URL}/home`);
    } catch (err: any) {
      console.error('OAuth callback error:', err);
      // Redirect to frontend with error
      const frontendErrorUrl = new URL('/login', process.env.FRONTEND_URL!);
      frontendErrorUrl.searchParams.set('error', err.message || 'Authentication failed');
      return res.redirect(frontendErrorUrl.toString());
    }
  }

  @Post('verify')
  async verifyToken(@Req() req: Request) {
    const sessionToken = req.cookies?.__session;
    if (!sessionToken) {
      throw new UnauthorizedException('No session token found');
    }
    
    try {
      const decodedToken = await this.authService.verifyToken(sessionToken);
      // Get user role to return complete user info
      const userRole = await this.authService.getUserRole(decodedToken.uid);
      
      return {
        valid: true,
        user: {
          uid: decodedToken.uid,
          email: decodedToken.email,
          role: userRole.role
        }
      };
    } catch (error) {
      throw new UnauthorizedException('Invalid token');
    }
  }

  @Post('roles/:userId')
  async assignUserRole(
    @Param('userId') userId: string,
    @Body('role') role: UserRole
  ) {
    if (!userId || !role) {
      throw new BadRequestException('User ID and role are required');
    }

    await this.authService.assignUserRole(userId, role);
    return { success: true };
  }

  @Get('roles/:userId')
  async getUserRole(@Param('userId') userId: string) {
    if (!userId) {
      throw new BadRequestException('User ID is required');
    }

    return await this.authService.getUserRole(userId);
  }

  @Get('roles')
  async listUsers(@Query('maxResults') maxResults?: string) {
    return await this.authService.listUsers(maxResults ? parseInt(maxResults) : undefined);
  }

  @Post('logout')
  async logout(@Res() res: Response) {
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/'
    };

    // Clear all auth-related cookies
    res.clearCookie('__session', cookieOptions);
    res.clearCookie('uid', cookieOptions);
    res.clearCookie('email', cookieOptions);
    res.clearCookie('role', cookieOptions);

    return res.json({ success: true });
  }
}
