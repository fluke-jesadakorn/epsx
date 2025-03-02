import { IsEnum, IsOptional } from 'class-validator';
import { ApiProperty } from '@nestjs/swagger';

export enum UserRole {
  PUBLIC = 'public',
  BASIC = 'basic',
  PREMIUM = 'premium',
}

export class GetEpsGrowthDto {
  @ApiProperty({
    required: false,
    type: Number,
    description: 'Number of records to return',
    example: 10,
  })
  @IsOptional()
  limit?: number;

  @ApiProperty({
    required: false,
    type: Number,
    description: 'Number of records to skip',
    example: 0,
  })
  @IsOptional()
  skip?: number;

  @ApiProperty({
    required: false,
    enum: UserRole,
    description: 'User role for access control',
    example: UserRole.BASIC,
  })
  @IsEnum(UserRole)
  @IsOptional()
  role?: UserRole = UserRole.PUBLIC;
}
