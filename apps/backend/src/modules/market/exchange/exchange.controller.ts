import { Controller, Get } from '@nestjs/common';
import { ApiTags } from '@nestjs/swagger';
import { ExchangeService } from './exchange.service';

@ApiTags('Exchange')
@Controller('exchange')
export class ExchangeController {
  constructor(private readonly exchangeService: ExchangeService) {}

  @Get()
  async getExchangeData() {
    return this.exchangeService.getExchangeData();
  }
}
