import { IsString, IsIn } from 'class-validator';

export class OAuthInitDTO {
  @IsString()
  @IsIn(['google'])
  provider!: string;
}

export class OAuthCallbackDTO {
  @IsString()
  code!: string;

  @IsString()
  state!: string;
}
