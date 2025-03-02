import { Controller, Post, Get, Body, Param, UseGuards, Query } from '@nestjs/common';
import { ApiOperation, ApiResponse, ApiTags, ApiQuery } from '@nestjs/swagger';
import { AuthService } from './auth.service';
import { AssignRoleDto } from './dto/assign-role.dto';
import { AdminGuard } from './guards/admin.guard';

@ApiTags('Auth')
@Controller('auth')
@UseGuards(AdminGuard)
export class AuthController {
  constructor(private readonly authService: AuthService) {}

  @Post('roles/assign')
  @ApiOperation({
    summary: 'Assign role to user',
    description: 'Admin endpoint to assign a role to a user'
  })
  @ApiResponse({
    status: 200,
    description: 'Role successfully assigned'
  })
  @ApiResponse({
    status: 403,
    description: 'Forbidden - Requires admin privileges'
  })
  @ApiResponse({
    status: 404,
    description: 'User not found'
  })
  async assignRole(@Body() assignRoleDto: AssignRoleDto): Promise<void> {
    await this.authService.assignUserRole(assignRoleDto.userId, assignRoleDto.role);
  }

  @Get('roles/:userId')
  @ApiOperation({
    summary: 'Get user role',
    description: 'Admin endpoint to get a user\'s role'
  })
  @ApiResponse({
    status: 200,
    description: 'User role retrieved successfully'
  })
  @ApiResponse({
    status: 403,
    description: 'Forbidden - Requires admin privileges'
  })
  @ApiResponse({
    status: 404,
    description: 'User not found'
  })
  async getUserRole(@Param('userId') userId: string) {
    return this.authService.getUserRole(userId);
  }

  @Get('roles')
  @ApiOperation({
    summary: 'List users with roles',
    description: 'Admin endpoint to list all users with their roles'
  })
  @ApiQuery({
    name: 'maxResults',
    required: false,
    type: Number,
    description: 'Maximum number of users to return',
  })
  @ApiResponse({
    status: 200,
    description: 'Users list retrieved successfully'
  })
  @ApiResponse({
    status: 403,
    description: 'Forbidden - Requires admin privileges'
  })
  async listUsers(@Query('maxResults') maxResults?: number) {
    return this.authService.listUsers(maxResults);
  }
}
