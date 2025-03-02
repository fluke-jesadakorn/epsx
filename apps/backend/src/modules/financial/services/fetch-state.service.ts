import { Injectable } from "@nestjs/common";

@Injectable()
export class FetchStateService {
  private lastProcessedStock: string | null = null;

  setLastProcessedStock(symbol: string) {
    this.lastProcessedStock = symbol;
  }

  getLastProcessedStock(): string | null {
    return this.lastProcessedStock;
  }

  resetLastProcessedStock() {
    this.lastProcessedStock = null;
  }
}
