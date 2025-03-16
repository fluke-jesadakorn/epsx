import WebSocket from 'ws';
import { 
  PriceUpdateCallback, 
  SessionMessage, 
  SubscriptionOptions, 
  SubscriptionOptionsInternal, 
  TradingViewMessage,
  QuoteMessage,
  QuoteData
} from './tv-websocket.types';

export class TradingViewWebSocket {
  private ws!: WebSocket;
  private nextMessageId = 1;
  private url: string;
  private sessionId = '';
  private chartSession = '';
  private quoteSession = '';
  private connected = false;
  private subscriptions = new Map<string, PriceUpdateCallback>();
  private pendingSubscriptions = new Map<string, SubscriptionOptionsInternal>();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private readyPromise: Promise<void>;
  private readyResolve!: () => void;
  private readyReject!: (error: Error) => void;
  private readyTimeoutId?: NodeJS.Timeout;
  private readonly READY_TIMEOUT = 30000;
  private sendQueue: Array<{ func: string; args: any[] }> = [];
  private isProcessingQueue = false;
  private readonly SEND_DELAY = 100;
  private debug: boolean;

  constructor(url = 'wss://data.tradingview.com/socket.io/websocket', debug = true) {
    this.url = url;
    this.debug = debug;
    this.readyPromise = new Promise((resolve, reject) => {
      this.readyResolve = resolve;
      this.readyReject = reject;
    });
    this.initWebSocket();
  }

  private log(...args: any[]): void {
    if (this.debug) {
      console.log('[TradingView WS]', ...args);
    }
  }

  public async waitForReady(): Promise<void> {
    clearTimeout(this.readyTimeoutId);
    return Promise.race([
      this.readyPromise,
      new Promise<void>((_, reject) => {
        this.readyTimeoutId = setTimeout(() => {
          const error = new Error('WebSocket connection timeout - failed to initialize within 30 seconds');
          this.readyReject(error);
          reject(error);
        }, this.READY_TIMEOUT);
      }),
    ]).finally(() => clearTimeout(this.readyTimeoutId));
  }

  private initWebSocket(): void {
    this.ws = new WebSocket(this.url, {
      headers: {
        Origin: 'https://www.tradingview.com',
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
      },
      handshakeTimeout: 10000
    });

    this.ws.on('open', () => {
      this.log('WebSocket connection opened');
      this.reconnectAttempts = 0;
      this.connected = false;
      this.sessionId = this.chartSession = this.quoteSession = '';
      this.send('set_auth_token', ['unauthorized_user_token']);
      this.send('quote_create_session', ['qs_test']);
      this.send('chart_create_session', ['cs_test', '']);
    });

    this.ws.on('message', (data: WebSocket.Data) => {
      try {
        const messages = this.parseMessages(data.toString());
        this.log('Received messages:', messages);
        messages.forEach(msg => this.handleMessage(msg));
      } catch (e) {
        console.error('Error processing message:', e);
      }
    });

    this.ws.on('error', (error: Error) => {
      console.error('WebSocket error:', error);
      this.cleanup();
      this.readyReject(error);
    });

    this.ws.on('close', (code: number, reason: string) => {
      this.log(`WebSocket closed with code ${code}:`, reason);
      this.cleanup();
      this.handleReconnect();
    });
  }

  private parseMessages(message: string): TradingViewMessage[] {
    this.log('Parsing message:', message);
    const messages: TradingViewMessage[] = [];
    const parts = message.split(/(~m~\d+~m~)/).filter(Boolean);

    for (let i = 0; i < parts.length; i++) {
      const frameMatch = parts[i].match(/~m~(\d+)~m~/);
      if (!frameMatch) continue;

      const payload = parts[++i];
      if (!payload || payload.length !== parseInt(frameMatch[1], 10)) continue;

      if (payload.includes('~h~')) {
        messages.push({ m: 'heartbeat', p: [payload.split('~h~')[1]] });
        continue;
      }

      try {
        const parsed = JSON.parse(payload);
        if (typeof parsed === 'object' && parsed !== null) {
          messages.push(parsed);
        }
      } catch (e) {
        this.log('Failed to parse message payload:', payload);
      }
    }
    return messages;
  }

  private async send(func: string, args: any[]): Promise<void> {
    this.sendQueue.push({ func, args });
    if (!this.isProcessingQueue) await this.processSendQueue();
  }

  private async processSendQueue(): Promise<void> {
    if (this.isProcessingQueue) return;
    this.isProcessingQueue = true;

    while (this.sendQueue.length > 0) {
      if (this.ws.readyState !== WebSocket.OPEN) break;

      const msg = this.sendQueue.shift();
      if (!msg) continue;

      try {
        const jsonStr = JSON.stringify({ m: msg.func, p: msg.args });
        const msgLen = Buffer.from(jsonStr).length;
        const formattedMsg = `~m~${msgLen}~m~${jsonStr}`;
        this.log('Sending message:', formattedMsg);
        this.ws.send(formattedMsg);
        await new Promise(resolve => setTimeout(resolve, this.SEND_DELAY));
      } catch (e) {
        console.error('Error sending message:', e);
        this.sendQueue.unshift(msg);
        await this.reconnect();
        break;
      }
    }
    
    this.isProcessingQueue = false;
  }

  public async subscribe(options: SubscriptionOptions): Promise<boolean> {
    if (!this.connected || !this.sessionId) {
      throw new Error('WebSocket not ready. Call waitForReady() first.');
    }

    if (this.isSubscribed(options.symbol)) {
      this.log(`Already subscribed to ${options.symbol}`);
      return false;
    }

    const { exchange, ticker } = this.parseSymbol(options.symbol);
    const symbolId = `sds_${ticker}`;
    const seriesId = `s_${ticker}`;
    const fullSymbol = `${exchange}:${ticker}`;

    this.log(`Subscribing to ${fullSymbol} (ID: ${symbolId})`);

    this.pendingSubscriptions.set(options.symbol, {
      ...options,
      symbolId,
      seriesId,
      exchange
    });

    if (options.onUpdate) {
      this.subscriptions.set(options.symbol, options.onUpdate);
    }

    try {
      await this.send('quote_add_symbols', [this.quoteSession, symbolId, fullSymbol]);
      await this.send('resolve_symbol', [
        this.chartSession,
        symbolId,
        JSON.stringify({
          symbol: fullSymbol,
          adjustment: 'splits',
          session: 'extended',
          type: 'stock'
        })
      ]);
      return true;
    } catch (error) {
      this.log(`Failed to subscribe to ${fullSymbol}:`, error);
      this.pendingSubscriptions.delete(options.symbol);
      this.subscriptions.delete(options.symbol);
      throw error;
    }
  }

  private handleMessage(msg: TradingViewMessage): void {
    this.log('Handling message:', msg);

    const handlers: Record<string, (msg: TradingViewMessage) => void> = {
      heartbeat: msg => this.sendPong(msg.p[0]),
      qsd: msg => this.handleQuoteData(msg),
      protocol_error: () => {
        this.log('Protocol error received');
        this.cleanup();
        this.reconnect();
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

    if (this.isSessionMessage(msg)) {
      this.sessionId = msg.session_id;
      this.connected = true;
      this.initializeSession();
      this.readyResolve();
    } else if (msg.m === 'symbol_resolved') {
      this.handleSymbolResolved(msg);
    } else if (msg.m === 'timescale_update' || msg.m === 'du') {
      this.handlePriceUpdate(msg.p);
    } else if (handlers[msg.m]) {
      handlers[msg.m](msg);
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

  private parseSymbol(symbol: string): { exchange: string; ticker: string } {
    const parts = symbol.split(':');
    if (parts.length === 2) return { exchange: parts[0], ticker: parts[1] };
    if (/^[A-Z]+$/.test(symbol)) return { exchange: 'NASDAQ', ticker: symbol };
    if (symbol.startsWith('SET_DLY:')) return { exchange: 'SET', ticker: symbol.substring(8) };
    return { exchange: 'SET_DLY', ticker: symbol };
  }

  private handleSymbolResolved(msg: TradingViewMessage): void {
    const symbolData = msg.p[1];
    if (symbolData?.symbol && this.pendingSubscriptions.has(symbolData.symbol)) {
      const subOptions = this.pendingSubscriptions.get(symbolData.symbol)!;
      this.log(`Symbol resolved: ${symbolData.symbol}`);
      this.pendingSubscriptions.delete(symbolData.symbol);
      this.send('create_series', [
        this.chartSession,
        subOptions.seriesId,
        subOptions.seriesId,
        subOptions.symbolId,
        subOptions.interval || '1D',
        300,
        ''
      ]);
    }
  }

  private isSessionMessage(msg: any): msg is SessionMessage {
    return msg?.session_id && typeof msg.session_id === 'string' &&
           'timestamp' in msg && 'timestampMs' in msg;
  }

  private initializeSession(): void {
    if (!this.connected || !this.sessionId) {
      this.readyReject(new Error('Cannot initialize session - not connected or no session ID'));
      return;
    }

    const generateId = () => Math.random().toString(36).substring(2, 15);
    this.chartSession = `cs_${generateId()}`;
    this.quoteSession = `qs_${generateId()}`;

    this.log('Initializing session', {
      chartSession: this.chartSession,
      quoteSession: this.quoteSession
    });

    this.send('chart_create_session', [this.chartSession, '']);
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

  private async handleReconnect(): Promise<void> {
    this.cleanup();
    if (this.reconnectAttempts++ < this.maxReconnectAttempts) {
      this.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
      await new Promise(resolve => setTimeout(resolve, this.reconnectDelay));
      this.reconnectDelay *= 2;
      this.connected = false;
      this.sessionId = this.chartSession = this.quoteSession = '';
      this.readyPromise = new Promise((resolve, reject) => {
        this.readyResolve = resolve;
        this.readyReject = reject;
      });
      this.initWebSocket();
    } else {
      const error = new Error('Max reconnection attempts reached');
      console.error(error.message);
      this.readyReject(error);
    }
  }

  private async reconnect(): Promise<void> {
    if (this.ws.readyState === WebSocket.OPEN) this.ws.close();
    await this.handleReconnect();
  }

  private isSubscribed(symbol: string): boolean {
    return this.subscriptions.has(symbol) || this.pendingSubscriptions.has(symbol);
  }

  private handlePriceUpdate(data: any[]): void {
    this.log('Price update received:', data);
    const seriesData = data[1];
    if (!seriesData?.s || !seriesData.v?.[0]) return;
    
    const symbol = seriesData.s.split('_')[1];
    const callback = this.subscriptions.get(symbol);
    if (callback) {
      const [timestamp, price, volume] = seriesData.v[0];
      callback({ 
        symbol, 
        price, 
        volume: volume || 0, 
        timestamp 
      });
    }
  }

  private sendPong(pingNumber: string): void {
    try {
      if (this.ws.readyState === WebSocket.OPEN) {
        const msg = `~m~${Buffer.from(`~h~${pingNumber}`).length}~m~~h~${pingNumber}`;
        this.ws.send(msg);
      }
    } catch (error) {
      console.error('Error sending pong:', error);
    }
  }

  private cleanup(): void {
    this.log('Cleaning up...');
    this.pendingSubscriptions.clear();
    clearTimeout(this.readyTimeoutId);
  }

  public unsubscribe(symbol: string): void {
    this.log(`Unsubscribing from ${symbol}`);
    this.pendingSubscriptions.delete(symbol);
    if (this.subscriptions.has(symbol)) {
      this.send('remove_series', [this.chartSession, `s_${symbol}`]);
      this.subscriptions.delete(symbol);
    }
  }

  public disconnect(): void {
    this.log('Disconnecting...');
    this.cleanup();
    if (this.connected) {
      this.ws.close();
      this.connected = false;
      this.sessionId = this.chartSession = this.quoteSession = '';
      this.subscriptions.clear();
      this.pendingSubscriptions.clear();
      this.readyPromise = new Promise((resolve, reject) => {
        this.readyResolve = resolve;
        this.readyReject = reject;
      });
    }
  }

  public destroy(): void {
    this.log('Destroying instance...');
    this.disconnect();
    if (this.ws) {
      this.ws.removeAllListeners();
      if (this.ws.readyState === WebSocket.OPEN) {
        try {
          this.ws.terminate();
        } catch (error) {
          console.error('Error terminating WebSocket:', error);
        }
      }
    }
  }
}

export default TradingViewWebSocket;
