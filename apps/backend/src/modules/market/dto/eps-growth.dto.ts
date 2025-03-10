import { IsEnum, IsOptional } from 'class-validator';
import { ApiProperty } from '@nestjs/swagger';
import { UserRole } from '../../../shared/types/roles.enum';

export class GetEpsGrowthDto {
  @ApiProperty({
    description: 'Maximum number of EPS growth records to return in a single request',
    type: 'number',
    minimum: 1,
    maximum: 100,
    default: 10,
    required: false,
    example: 20
  })
  @IsOptional()
  limit?: number;

  @ApiProperty({
    description: 'Number of records to skip for pagination. Use with limit to iterate through large datasets',
    type: 'number',
    minimum: 0,
    default: 0,
    required: false,
    example: 20
  })
  @IsOptional()
  skip?: number;

  @ApiProperty({
    description: 'User role that determines access level and data visibility. Access levels:\n' +
      '- guest: Basic EPS data with delays\n' +
      '- registered_user: Standard EPS data access\n' +
      '- premium_user: Real-time EPS data\n' +
      '- token_holder: Advanced EPS analytics\n' +
      '- administrator: Full data access',
    enum: UserRole,
    default: UserRole.GUEST,
    required: false,
    example: UserRole.PREMIUM_USER,
    enumName: 'UserRole',
    examples: Object.values(UserRole)
  })
  @IsEnum(UserRole)
  @IsOptional()
  role?: UserRole = UserRole.GUEST;
}
