import { IsString, IsEnum } from 'class-validator';
import { ApiProperty } from '@nestjs/swagger';
import { UserRole } from '../../../shared/types/roles.enum';

export class AssignRoleDto {
  @ApiProperty({
    description: 'Firebase unique user identifier',
    example: 'fMWEP12kX5WSHDBpYq9MMsZ3ale2',
    type: 'string',
    minLength: 20,
    maxLength: 36,
    pattern: '^[A-Za-z0-9]+$'
  })
  @IsString()
  userId!: string;

  @ApiProperty({
    description: 'Role to assign to the user. Available roles:\n' +
      '- guest: Basic access with limited features\n' +
      '- registered_user: Standard access after registration\n' +
      '- premium_user: Enhanced access with premium features\n' +
      '- token_holder: Full access with token holder benefits\n' +
      '- administrator: Complete system access and management',
    enum: UserRole,
    example: UserRole.REGISTERED_USER,
    default: UserRole.REGISTERED_USER,
    enumName: 'UserRole',
    examples: Object.values(UserRole)
  })
  @IsEnum(UserRole)
  role!: UserRole;
}
