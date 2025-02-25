import { ApiProperty } from "@nestjs/swagger";

// Base DTO with common fields
class BaseDto {
  @ApiProperty({ description: "Created by user ID", required: false })
  create_by?: string;

  @ApiProperty({ description: "Last edited by user ID", required: false })
  edit_by?: string;

  @ApiProperty({ description: "Deleted by user ID", required: false })
  delete_by?: string;

  @ApiProperty({
    description: "Document version for optimistic locking",
    default: 1,
  })
  version?: number;
}

// Exchange DTOs
export class CreateExchangeDto extends BaseDto {
  @ApiProperty({ description: "Unique market code identifier", example: "SET" })
  market_code: string = "";

  @ApiProperty({
    description: "Full name of the exchange",
    example: "Stock Exchange of Thailand",
  })
  exchange_name: string = "";

  @ApiProperty({
    description: "Description of the exchange",
    required: false,
    example: "Thailand's main stock exchange",
  })
  description?: string;

  @ApiProperty({
    description: "Geographic region of the exchange",
    example: "Asia",
    required: false,
  })
  region?: string;

  @ApiProperty({
    description: "Timezone of the exchange",
    example: "Asia/Bangkok",
    required: false,
  })
  timezone?: string;

  @ApiProperty({
    description: "Current market status",
    example: "OPEN",
    enum: ["OPEN", "CLOSED", "PRE_MARKET", "AFTER_HOURS"],
  })
  market_status: string = "CLOSED";

  @ApiProperty({
    description: "Regular trading hours start time",
    example: "10:00",
    required: false,
  })
  trading_hours_start?: string;

  @ApiProperty({
    description: "Regular trading hours end time",
    example: "16:30",
    required: false,
  })
  trading_hours_end?: string;

  @ApiProperty({
    description: "Trading days",
    example: ["MONDAY", "TUESDAY", "WEDNESDAY", "THURSDAY", "FRIDAY"],
    required: false,
  })
  trading_days?: string[];

  @ApiProperty({
    description: "Base currency of the exchange",
    example: "THB",
    required: false,
  })
  currency?: string;
}

export class ExchangeResponseDto extends CreateExchangeDto {
  @ApiProperty({
    description: "List of stock IDs associated with this exchange",
  })
  stocks: string[] = [];
}

// Stock DTOs
export class CreateStockDto extends BaseDto {
  @ApiProperty({ description: "Stock ticker symbol", example: "AOT" })
  symbol: string = "";

  @ApiProperty({
    description: "Company name",
    example: "Airports of Thailand PCL",
    required: false,
  })
  company_name?: string;

  @ApiProperty({ description: "Exchange ID this stock belongs to" })
  exchange: string = "";

  @ApiProperty({
    description: "Business sector",
    example: "Transportation",
    required: false,
  })
  sector?: string;

  @ApiProperty({
    description: "Specific industry",
    example: "Airport Services",
    required: false,
  })
  industry?: string;

  @ApiProperty({
    description: "Market capitalization in base currency",
    example: 634500000000,
    minimum: 0,
    required: false,
  })
  market_cap?: number;

  @ApiProperty({
    description: "Company website URL",
    example: "https://www.airportthai.co.th",
    required: false,
  })
  website?: string;

  @ApiProperty({
    description: "Current stock price in exchange currency",
    example: 44.25,
    minimum: 0,
    required: false,
  })
  current_price?: number;

  @ApiProperty({
    description: "Trading volume for the current day",
    example: 15678900,
    minimum: 0,
    required: false,
  })
  volume?: number;

  @ApiProperty({
    description: "Price change percentage for the current day",
    example: 2.5,
    required: false,
  })
  price_change_percent?: number;

  @ApiProperty({
    description: "Last updated timestamp",
    example: "2024-02-25T15:30:00Z",
    required: false,
  })
  last_updated?: Date;
}

export class StockResponseDto extends CreateStockDto {
  @ApiProperty({
    description: "List of financial report IDs associated with this stock",
  })
  financial: string[] = [];
}

// Financial DTOs
export class FinancialReportDto extends BaseDto {
  @ApiProperty({ description: "Associated stock ID" })
  stock: string = "";

  @ApiProperty({ description: "Report date", example: "2024-12-31" })
  report_date: Date = new Date();

  @ApiProperty({
    description: "Fiscal quarter (1-4)",
    example: 4,
    minimum: 1,
    maximum: 4,
  })
  fiscal_quarter: number = 0;

  @ApiProperty({ description: "Fiscal year", example: 2024, minimum: 1900 })
  fiscal_year: number = 0;

  @ApiProperty({
    description: "Total revenue in base currency",
    example: 12345678900,
    minimum: 0,
  })
  revenue?: number;

  @ApiProperty({
    description: "Year-over-year revenue growth percentage",
    example: 15.5,
  })
  revenue_growth?: number;

  @ApiProperty({
    description: "Net income in base currency",
    example: 2345678900,
    minimum: -999999999999,
  })
  net_income?: number;

  @ApiProperty({ description: "Net income margin percentage", example: 19.0 })
  net_income_margin?: number;

  @ApiProperty({
    description: "Operating income in base currency",
    example: 3456789000,
    minimum: -999999999999,
  })
  operating_income?: number;

  @ApiProperty({ description: "Operating margin percentage", example: 28.0 })
  operating_margin?: number;

  @ApiProperty({ description: "Basic earnings per share", example: 2.5 })
  eps_basic?: number;

  @ApiProperty({ description: "Diluted earnings per share", example: 2.45 })
  eps_diluted?: number;

  @ApiProperty({
    description: "Year-over-year EPS growth percentage",
    example: 12.3,
  })
  eps_growth?: number;

  @ApiProperty({
    description: "EBITDA value in base currency",
    example: 4567890000,
    minimum: -999999999999,
  })
  ebitda?: number;

  @ApiProperty({ description: "EBITDA margin percentage", example: 37.0 })
  ebitda_margin?: number;

  @ApiProperty({
    description: "Total assets in base currency",
    example: 78901234500,
    minimum: 0,
  })
  total_assets?: number;

  @ApiProperty({
    description: "Total liabilities in base currency",
    example: 45678901200,
    minimum: 0,
  })
  total_liabilities?: number;

  @ApiProperty({
    description: "Total equity in base currency",
    example: 33222333300,
    minimum: -999999999999,
  })
  total_equity?: number;
}

// Query DTOs
export class PaginationQueryDto {
  @ApiProperty({
    description: "Number of records to skip",
    required: false,
    minimum: 0,
  })
  skip?: number;

  @ApiProperty({
    description: "Number of records to return",
    required: false,
    minimum: 1,
    maximum: 100,
  })
  limit?: number;
}

export class ScrapeStocksDto {
  @ApiProperty({
    description:
      "List of exchange IDs to scrape stocks from. If empty, scrapes from all exchanges.",
    required: false,
    type: [String],
  })
  exchangeIds?: string[];
}

// Response DTOs
export class EPSGrowthResponseDto {
  @ApiProperty({ description: "Stock symbol", example: "AOT" })
  symbol: string = "";

  @ApiProperty({
    description: "Company name",
    example: "Airports of Thailand PCL",
  })
  company_name: string = "";

  @ApiProperty({ description: "EPS growth percentage", example: 15.5 })
  eps_growth: number = 0;

  @ApiProperty({ description: "Latest report date", example: "2024-12-31" })
  report_date: Date = new Date();
}

export class ScrapingResponseDto {
  @ApiProperty({ description: "Number of records processed", example: 100 })
  processed: number = 0;

  @ApiProperty({
    description: "Number of records successfully updated",
    example: 95,
  })
  success: number = 0;

  @ApiProperty({ description: "Number of records that failed", example: 5 })
  failed: number = 0;

  @ApiProperty({
    description: "Error messages if any",
    required: false,
    type: [String],
  })
  errors?: string[];
}

// Error DTOs
export class MarketErrorResponseDto {
  @ApiProperty({ description: "Error status code", example: 404 })
  statusCode: number = 500;

  @ApiProperty({ description: "Error message", example: "Stock not found" })
  message: string = "";

  @ApiProperty({
    description: "Error description",
    example: "The requested stock symbol does not exist",
  })
  error: string = "";
}
