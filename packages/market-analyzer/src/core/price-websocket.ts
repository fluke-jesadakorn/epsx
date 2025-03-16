import { BaseWebSocket } from './base-websocket';
import {
  PriceUpdateCallback,
  SubscriptionOptions,
  SubscriptionOptionsInternal,
  TradingViewMessage,
  QuoteMessage
} from '../types';
import { ERROR_MESSAGES } from '../utils/constants';
import { formatSymbol, generateSessionId } from '../utils/helpers';

export class PriceWebSocket extends BaseWebSocket {
  private quoteSession = '';
  private subscriptions = new Map<string, PriceUpdateCallback>();
  private pendingSubscriptions = new Map<string, SubscriptionOptionsInternal>();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  protected override onOpen(): void {
    this.reconnectAttempts = 0;
    this.quoteSession = '';
    this.send('quote_create_session', ['qs_test']);
  }

  protected override initializeSession(): void {
    if (!this.connected || !this.sessionId) {
      this.readyReject(new Error('Cannot initialize session - not connected or no session ID'));
      return;
    }

    this.quoteSession = `qs_${generateSessionId()}`;

    this.log('Initializing session', {
      quoteSession: this.quoteSession
    });

    this.send('quote_create_session', [this.quoteSession]);

    if (this.subscriptions.size > 0) {
      Promise.all(
        Array.from(this.subscriptions.entries()).map(([symbol, callback]) =>
          this.subscribe({ symbol, onUpdate: callback }).catch(() => {
            this.subscriptions.delete(symbol);
            return false;
          })
        )
      );
    }
  }

  protected override onMessage(msg: TradingViewMessage): void {
    const handlers: Record<string, (msg: TradingViewMessage) => void> = {
      heartbeat: msg => this.sendPong(msg.p[0]),
      qsd: msg => this.handleQuoteData(msg),
      protocol_error: () => {
        this.log('Protocol error received');
        this.cleanup();
        this.handleReconnect();
      },
      symbol_error: msg => {
        const [, symbolId] = msg.p;
        if (symbolId?.startsWith('sds_')) {
          const symbol = symbolId.substring(4);
          const subscription = Array.from(this.pendingSubscriptions.entries())
            .find(([, options]) => options.symbolId === symbolId);
          if (subscription) {
            const [originalSymbol] = subscription;
            this.log(`Symbol error for ${originalSymbol}`);
            this.pendingSubscriptions.delete(originalSymbol);
            this.subscriptions.delete(originalSymbol);
          }
        }
      }
    };

    if (handlers[msg.m]) {
      handlers[msg.m](msg);
    }
  }

  protected override onCleanup(): void {
    this.pendingSubscriptions.clear();
  }

  protected override onClose(code: number, reason: string): void {
    this.handleReconnect();
  }

  public async subscribe(options: SubscriptionOptions): Promise<boolean> {
    if (!this.connected || !this.sessionId) {
      throw new Error(ERROR_MESSAGES.WS_NOT_READY);
    }

    const formattedSymbol = formatSymbol(options.symbol);
    if (this.isSubscribed(formattedSymbol)) {
      this.log(`Already subscribed to ${formattedSymbol}`);
      return false;
    }

    const [exchange, ticker] = formattedSymbol.split(':');
    const symbolId = `sds_${ticker}`;
    const seriesId = `s_${ticker}`;

    this.log(`Subscribing to ${formattedSymbol} (ID: ${symbolId})`);

    this.pendingSubscriptions.set(formattedSymbol, {
      ...options,
      symbol: formattedSymbol,
      symbolId,
      seriesId
    });

    if (options.onUpdate) {
      this.subscriptions.set(formattedSymbol, options.onUpdate);
    }

    try {
      await this.send('quote_add_symbols', [this.quoteSession, symbolId, formattedSymbol]);
      return true;
    } catch (error) {
      this.log(`Failed to subscribe to ${formattedSymbol}:`, error);
      this.pendingSubscriptions.delete(formattedSymbol);
      this.subscriptions.delete(formattedSymbol);
      throw error;
    }
  }

  private handleQuoteData(msg: TradingViewMessage): void {
    try {
      const [, data] = msg.p as [unknown, QuoteMessage];
      if (!data || typeof data !== 'object') return;

      for (const [key, quoteData] of Object.entries(data)) {
        if (!key.startsWith('sds_')) continue;

        const symbol = key.substring(4);
        const callback = this.subscriptions.get(symbol);
        if (callback && quoteData) {
          callback({
            symbol,
            price: quoteData.lp ?? 0,
            volume: quoteData.volume ?? 0,
            timestamp: Math.floor(Date.now() / 1000)
          });
        }
      }
    } catch (e) {
      console.error('Error handling quote data:', e);
    }
  }

  private async handleReconnect(): Promise<void> {
    if (this.reconnectAttempts++ < this.maxReconnectAttempts) {
      this.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
      await new Promise(resolve => setTimeout(resolve, this.reconnectDelay));
      this.reconnectDelay *= 2;
      this.connected = false;
      this.sessionId = this.quoteSession = '';
      this.readyPromise = new Promise((resolve, reject) => {
        this.readyResolve = resolve;
        this.readyReject = reject;
      });
      this.initWebSocket();
    } else {
      const error = new Error(ERROR_MESSAGES.MAX_RETRIES_REACHED);
      console.error(error.message);
      this.readyReject(error);
    }
  }

  private isSubscribed(symbol: string): boolean {
    return this.subscriptions.has(symbol) || this.pendingSubscriptions.has(symbol);
  }

  private sendPong(pingNumber: string): void {
    try {
      if (this.ws.readyState === WebSocket.OPEN) {
        this.send('~h~', [pingNumber]);
      }
    } catch (error) {
      console.error('Error sending pong:', error);
    }
  }

  public unsubscribe(symbol: string): void {
    const formattedSymbol = formatSymbol(symbol);
    this.log(`Unsubscribing from ${formattedSymbol}`);
    this.pendingSubscriptions.delete(formattedSymbol);
    if (this.subscriptions.has(formattedSymbol)) {
      this.subscriptions.delete(formattedSymbol);
    }
  }

  public override disconnect(): void {
    super.disconnect();
    this.subscriptions.clear();
    this.pendingSubscriptions.clear();
  }
}

export default PriceWebSocket;
