import { Controller, Get } from '@nestjs/common';
import { ApiTags } from '@nestjs/swagger';
import { FinancialService } from './financial.service';

@ApiTags('Financial')
@Controller('financial')
export class FinancialController {
  constructor(private readonly financialService: FinancialService) {}

  @Get()
  async getFinancialData() {
    return this.financialService.getFinancialData();
  }

  @Get('indicators')
  async getFinancialIndicators() {
    return this.financialService.getFinancialIndicators();
  }
}
