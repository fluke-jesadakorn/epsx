import { ApiProperty } from '@nestjs/swagger';

export class HealthCheckResponse {
  @ApiProperty({
    description: 'The status of the service',
    example: 'ok'
  })
  status!: string;

  @ApiProperty({
    description: 'The name of the service',
    example: 'market'
  })
  service!: string;
}
