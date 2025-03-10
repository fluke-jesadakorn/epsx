import {
  Controller,
  Get,
  Post,
  Body,
  Param,
  Query,
  UseGuards,
  BadRequestException,
} from "@nestjs/common";
import { UserRole } from "@epsx/shared/types/roles.enum";
import { Roles } from "@epsx/shared/decorators/roles.decorator";
import { FirebaseAuthGuard } from "@epsx/shared/guards/firebase-auth.guard";
import { RolesGuard } from "@epsx/shared/guards/roles.guard";
import { UserService, User } from "../services/user.service";
import {
  ApiBearerAuth,
  ApiBody,
  ApiOperation,
  ApiParam,
  ApiQuery,
  ApiResponse,
  ApiTags,
} from "@nestjs/swagger";

@ApiTags("Users")
@Controller("users")
@UseGuards(FirebaseAuthGuard, RolesGuard)
@ApiBearerAuth()
export class UserController {
  constructor(private readonly userService: UserService) {}

  @Get()
  @Roles(UserRole.ADMINISTRATOR)
  @ApiOperation({
    summary: "List all users",
    description: "Get list of all users with their roles (Administrator only)",
  })
  @ApiQuery({
    name: "maxResults",
    required: false,
    type: "number",
    description: "Maximum number of users to return",
  })
  @ApiResponse({
    status: 200,
    description: "List of users retrieved successfully",
    schema: {
      type: "object",
      properties: {
        users: {
          type: "array",
          items: {
            type: "object",
            properties: {
              uid: {
                type: "string",
                description: "Firebase user ID",
              },
              email: {
                type: "string",
                description: "User email address",
              },
              role: {
                type: "string",
                enum: Object.values(UserRole),
                description: "User role",
              },
              displayName: {
                type: "string",
                description: "User display name",
                nullable: true,
              },
              photoURL: {
                type: "string",
                description: "User profile photo URL",
                nullable: true,
              },
            },
          },
        },
        pageToken: {
          type: "string",
          description: "Token for retrieving the next page",
          nullable: true,
        },
      },
    },
  })
  @ApiResponse({
    status: 400,
    description: "Bad Request - Invalid maxResults parameter",
    schema: {
      type: "object",
      properties: {
        message: {
          type: "string",
          example: "maxResults must be a number",
        },
        error: {
          type: "string",
          example: "Bad Request",
        },
        statusCode: {
          type: "number",
          example: 400,
        },
      },
    },
  })
  @ApiResponse({
    status: 403,
    description: "Forbidden - requires administrator role",
  })
  async listUsers(@Query("maxResults") maxResults?: string): Promise<User[]> {
    const limit = maxResults ? parseInt(maxResults, 10) : undefined;

    if (maxResults && isNaN(limit!)) {
      throw new BadRequestException("maxResults must be a number");
    }

    return this.userService.listUsers(limit);
  }

  @Post(":userId/role")
  @Roles(UserRole.ADMINISTRATOR)
  @ApiOperation({
    summary: "Update user role",
    description: "Assign a new role to a user (Administrator only)",
  })
  @ApiParam({
    name: "userId",
    required: true,
    type: "string",
    description: "Firebase user ID",
  })
  @ApiBody({
    schema: {
      type: "object",
      required: ["role"],
      properties: {
        role: {
          type: "string",
          enum: Object.values(UserRole),
          description: "New role to assign to the user",
        },
      },
    },
  })
  @ApiResponse({
    status: 200,
    description: "Role updated successfully",
    schema: {
      type: "object",
      properties: {
        success: {
          type: "boolean",
          example: true,
        },
      },
    },
  })
  @ApiResponse({
    status: 400,
    description: "Invalid role or user ID",
    schema: {
      type: "object",
      properties: {
        message: {
          type: "string",
          example: "Invalid role",
        },
        error: {
          type: "string",
          example: "Bad Request",
        },
        statusCode: {
          type: "number",
          example: 400,
        },
      },
    },
  })
  @ApiResponse({
    status: 403,
    description: "Forbidden - requires administrator role",
  })
  async updateUserRole(
    @Param("userId") userId: string,
    @Body("role") role: UserRole
  ) {
    if (!Object.values(UserRole).includes(role)) {
      throw new BadRequestException("Invalid role");
    }

    await this.userService.assignUserRole(userId, role);
    return { success: true };
  }
}
