import { BaseWebSocket } from './base-websocket';
import { TradingViewMessage } from '../types';
import { ERROR_MESSAGES } from '../utils/constants';
import { formatSymbol, generateSessionId } from '../utils/helpers';

export interface FinancialData {
  symbol: string;
  data: any;
}

export class FinancialsWebSocket extends BaseWebSocket {
  private chartSession = '';
  private dataResolver?: (value: FinancialData | PromiseLike<FinancialData>) => void;
  private currentSymbol?: string;

  protected override onOpen(): void {
    this.chartSession = '';
    this.send('chart_create_session', ['cs_test', '']);
  }

  protected override initializeSession(): void {
    if (!this.connected || !this.sessionId) {
      this.readyReject(new Error('Cannot initialize session - not connected or no session ID'));
      return;
    }

    this.chartSession = `cs_${generateSessionId()}`;

    this.log('Initializing session', {
      chartSession: this.chartSession
    });

    this.send('chart_create_session', [this.chartSession, '']);
  }

  protected override onMessage(msg: TradingViewMessage): void {
    if (msg.m === 'symbol_resolved' && this.currentSymbol && this.dataResolver) {
      const symbolData = msg.p[1];
      this.dataResolver({
        symbol: this.currentSymbol,
        data: symbolData
      });
      this.currentSymbol = undefined;
      this.dataResolver = undefined;
      this.disconnect();
    }
  }

  protected override onCleanup(): void {
    this.currentSymbol = undefined;
    this.dataResolver = undefined;
  }

  public async getFinancials(symbol: string): Promise<FinancialData> {
    if (!this.connected || !this.sessionId) {
      throw new Error(ERROR_MESSAGES.WS_NOT_READY);
    }

    const formattedSymbol = formatSymbol(symbol);
    const [exchange, ticker] = formattedSymbol.split(':');
    const symbolId = `sds_${ticker}`;

    this.log(`Requesting financials for ${formattedSymbol}`);

    this.currentSymbol = formattedSymbol;

    try {
      await this.send('resolve_symbol', [
        this.chartSession,
        symbolId,
        JSON.stringify({
          symbol: formattedSymbol,
          adjustment: 'splits',
          session: 'extended',
          type: 'stock'
        })
      ]);

      return new Promise((resolve) => {
        this.dataResolver = resolve;
      });
    } catch (error) {
      this.log(`Failed to get financials for ${formattedSymbol}:`, error);
      throw error;
    }
  }
}

export default FinancialsWebSocket;
