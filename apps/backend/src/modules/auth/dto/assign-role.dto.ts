import { IsString, IsEnum } from 'class-validator';
import { ApiProperty } from '@nestjs/swagger';
import { UserRole } from '../../../shared/guards/role.guard';

export class AssignRoleDto {
  @ApiProperty({
    description: 'Firebase user ID',
    example: 'abcd1234',
  })
  @IsString()
  userId!: string;

  @ApiProperty({
    description: 'Role to assign to the user',
    enum: UserRole,
    example: UserRole.BASIC,
  })
  @IsEnum(UserRole)
  role!: UserRole;
}
