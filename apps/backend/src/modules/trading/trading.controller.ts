import { Controller, Get, Post, Body, ForbiddenException } from '@nestjs/common';
import { ApiTags, ApiOperation, ApiBearerAuth } from '@nestjs/swagger';
import {
  RequirePortfolioManagement,
  RequireTradingBot,
  RequireAIAnalysis,
  RequirePortfolioAutomation,
  TokenFeature,
  RequireFeatures,
  PERMISSIONS,
  PermissionKey
} from '../auth/auth.module';

@ApiTags('Trading')
@Controller('trading')
@ApiBearerAuth()
export class TradingController {
  @ApiOperation({ summary: 'Get portfolio overview' })
  @RequirePortfolioManagement()
  @Get('portfolio/overview')
  async getPortfolioOverview() {
    return {
      message: 'Portfolio overview accessed'
    };
  }

  @ApiOperation({ summary: 'Execute trading bot' })
  @RequireTradingBot()
  @Post('bot/execute')
  async executeTradingBot(@Body() params: any) {
    return {
      message: 'Trading bot executed',
      params
    };
  }

  @ApiOperation({ summary: 'Get AI analysis' })
  @RequireAIAnalysis()
  @Get('analysis/ai')
  async getAIAnalysis() {
    return {
      message: 'AI analysis retrieved'
    };
  }

  @ApiOperation({ summary: 'Execute automated portfolio trade' })
  @RequirePortfolioAutomation()
  @Post('portfolio/auto-trade')
  async executePortfolioTrade(@Body() tradeParams: any) {
    return {
      message: 'Automated portfolio trade executed',
      params: tradeParams
    };
  }

  @ApiOperation({ summary: 'Get risk analysis' })
  @RequireFeatures(
    [TokenFeature.ADVANCED_TOOLS],
    [PERMISSIONS.VIEW_RISK_ANALYSIS as PermissionKey]
  )
  @Get('analysis/risk')
  async getRiskAnalysis() {
    return {
      message: 'Risk analysis retrieved'
    };
  }

  // Example of custom authorization handling
  @ApiOperation({ summary: 'Execute high-value trade' })
  @RequirePortfolioManagement()
  @Post('portfolio/high-value-trade')
  async executeHighValueTrade(@Body() tradeParams: { amount: number }) {
    // Additional custom authorization check
    if (tradeParams.amount > 100000) {
      throw new ForbiddenException(
        'High-value trades above 100,000 require additional verification'
      );
    }

    return {
      message: 'High-value trade executed',
      params: tradeParams
    };
  }
}
