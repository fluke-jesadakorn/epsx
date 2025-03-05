import { IsString, IsOptional } from 'class-validator';

export class GoogleSignInDTO {
  @IsString()
  idToken: string;

  @IsOptional()
  @IsString()
  redirectUrl?: string;
}
