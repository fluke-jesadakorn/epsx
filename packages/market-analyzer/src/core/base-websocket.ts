import WebSocket from 'ws';
import { 
  SessionMessage, 
  TradingViewMessage
} from '../types';
import { 
  ERROR_MESSAGES, 
  TRADINGVIEW_ENDPOINTS, 
  WS_CONFIG 
} from '../utils/constants';
import { 
  delay,
  parseWebSocketMessage, 
  formatWebSocketMessage 
} from '../utils/helpers';

export abstract class BaseWebSocket {
  protected ws!: WebSocket;
  protected url: string;
  protected sessionId = '';
  protected connected = false;
  protected readyPromise: Promise<void>;
  protected readyResolve!: () => void;
  protected readyReject!: (error: Error) => void;
  protected readyTimeoutId?: NodeJS.Timeout;
  protected readonly READY_TIMEOUT = WS_CONFIG.READY_TIMEOUT;
  protected sendQueue: Array<{ func: string; args: any[] }> = [];
  protected isProcessingQueue = false;
  protected readonly SEND_DELAY = WS_CONFIG.SEND_DELAY;
  protected debug: boolean;

  constructor(url = TRADINGVIEW_ENDPOINTS.WEBSOCKET, debug = true) {
    this.url = url;
    this.debug = debug;
    this.readyPromise = new Promise((resolve, reject) => {
      this.readyResolve = resolve;
      this.readyReject = reject;
    });
    this.initWebSocket();
  }

  protected log(...args: any[]): void {
    if (this.debug) {
      console.log(`[${this.constructor.name}]`, ...args);
    }
  }

  public async waitForReady(): Promise<void> {
    clearTimeout(this.readyTimeoutId);
    return Promise.race([
      this.readyPromise,
      new Promise<void>((_, reject) => {
        this.readyTimeoutId = setTimeout(() => {
          const error = new Error(ERROR_MESSAGES.WS_CONNECTION_TIMEOUT);
          this.readyReject(error);
          reject(error);
        }, this.READY_TIMEOUT);
      }),
    ]).finally(() => clearTimeout(this.readyTimeoutId));
  }

  protected initWebSocket(): void {
    this.ws = new WebSocket(this.url, {
      headers: {
        Origin: 'https://www.tradingview.com',
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
      },
      handshakeTimeout: 10000
    });

    this.ws.on('open', () => {
      this.log('WebSocket connection opened');
      this.connected = false;
      this.sessionId = '';
      this.send('set_auth_token', ['unauthorized_user_token']);
      this.onOpen();
    });

    this.ws.on('message', (data: WebSocket.Data) => {
      try {
        const messages = parseWebSocketMessage(data.toString());
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
      this.onError(error);
    });

    this.ws.on('close', (code: number, reason: string) => {
      this.log(`WebSocket closed with code ${code}:`, reason);
      this.cleanup();
      this.onClose(code, reason);
    });
  }

  protected async send(func: string, args: any[]): Promise<void> {
    this.sendQueue.push({ func, args });
    if (!this.isProcessingQueue) await this.processSendQueue();
  }

  protected async processSendQueue(): Promise<void> {
    if (this.isProcessingQueue) return;
    this.isProcessingQueue = true;

    while (this.sendQueue.length > 0) {
      if (this.ws.readyState !== WebSocket.OPEN) break;

      const msg = this.sendQueue.shift();
      if (!msg) continue;

      try {
        const formattedMsg = formatWebSocketMessage(msg.func, msg.args);
        this.log('Sending message:', formattedMsg);
        this.ws.send(formattedMsg);
        await delay(this.SEND_DELAY);
      } catch (e) {
        console.error('Error sending message:', e);
        this.sendQueue.unshift(msg);
        break;
      }
    }
    
    this.isProcessingQueue = false;
  }

  protected handleMessage(msg: TradingViewMessage): void {
    if (this.isSessionMessage(msg)) {
      this.sessionId = msg.session_id;
      this.connected = true;
      this.initializeSession();
      this.readyResolve();
    } else {
      this.onMessage(msg);
    }
  }

  protected isSessionMessage(msg: any): msg is SessionMessage {
    return msg?.session_id && typeof msg.session_id === 'string' &&
           'timestamp' in msg && 'timestampMs' in msg;
  }

  protected cleanup(): void {
    this.log('Cleaning up...');
    clearTimeout(this.readyTimeoutId);
    this.onCleanup();
  }

  public disconnect(): void {
    this.log('Disconnecting...');
    this.cleanup();
    if (this.connected) {
      this.ws.close();
      this.connected = false;
      this.sessionId = '';
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

  // Abstract methods that derived classes must implement
  protected abstract initializeSession(): void;
  protected abstract onMessage(msg: TradingViewMessage): void;
  protected abstract onCleanup(): void;

  // Optional hooks that derived classes can override
  protected onOpen(): void {}
  protected onError(error: Error): void {}
  protected onClose(code: number, reason: string): void {}
}

export default BaseWebSocket;
