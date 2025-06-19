use futures_util::{ SinkExt, StreamExt };
use reqwest::header::{ HeaderMap, HeaderValue };
use serde::{ de::DeserializeOwned, Serialize };
use tokio_tungstenite::{ connect_async, tungstenite::protocol::Message, WebSocketStream };
use tokio::sync::mpsc;
use url::Url;
use futures_util::stream::{ SplitSink, SplitStream };

use super::error::StockServiceError;
use tracing::{ debug, error, info };

#[allow(dead_code)]
pub struct WebSocketClient {
    url: String,
    auth_token: String,
    headers: HeaderMap,
}

impl WebSocketClient {
    pub fn new(url: &str, auth_token: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", HeaderValue::from_static("https://www.tradingview.com"));
        headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"));
        headers.insert("Accept-Language", HeaderValue::from_static("th-TH,th;q=0.9,en;q=0.8"));
        headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        headers.insert("Pragma", HeaderValue::from_static("no-cache"));
        headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br, zstd"));


        Self {
            url: url.to_string(),
            auth_token: auth_token.to_string(),
            headers,
        }
    }

    pub async fn connect<T>(&self) -> Result<WebSocketConnection<T>, StockServiceError>
        where T: DeserializeOwned + Send + std::fmt::Debug + 'static
    {
        let url = Url::parse(&self.url).map_err(|e|
            StockServiceError::WebSocketError(e.to_string())
        )?;

        let (ws_stream, _) = connect_async(&url).await.map_err(|e| {
            if e.to_string().contains("403 Forbidden") {
                StockServiceError::WebSocketError(
                    "TradingView authentication failed. Please ensure TRADINGVIEW_AUTH_TOKEN is properly set in your .env file. \
                    You can find this token in your TradingView session cookies.".to_string()
                )
            } else {
                StockServiceError::WebSocketError(e.to_string())
            }
        })?;

        info!("WebSocket connected to {}", self.url);

        let (tx, _rx) = mpsc::channel(100);
        let connection = WebSocketConnection::new(ws_stream, tx);

        // Send authentication token message
        let auth_message = format!("~m~54~m~{{\"m\":\"set_auth_token\",\"p\":[\"{}\"]}}", self.auth_token);
        connection.send_message(&auth_message).await?;

        Ok(connection)
    }
}

#[allow(dead_code)]
pub struct WebSocketConnection<T> {
    write: tokio::sync::Mutex<
        SplitSink<
            WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            Message
        >
    >,
    read: tokio::sync::Mutex<SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>,
    message_tx: mpsc::Sender<T>,
}

impl<T> WebSocketConnection<T> where T: DeserializeOwned + Send + std::fmt::Debug + 'static {
    fn new(
        ws_stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        message_tx: mpsc::Sender<T>
    ) -> Self {
        let (write, read) = ws_stream.split();
        Self {
            write: tokio::sync::Mutex::new(write),
            read: tokio::sync::Mutex::new(read),
            message_tx,
        }
    }

    pub async fn send_message<M: Serialize>(&self, message: &M) -> Result<(), StockServiceError> {
        let message = serde_json
            ::to_string(message)
            .map_err(|e| StockServiceError::ParserError(e))?;

        self.write
            .lock().await
            .send(Message::Text(message.clone())).await
            .map_err(|e| StockServiceError::WebSocketError(e.to_string()))?;

        debug!("Sent WebSocket message: {}", message);
        Ok(())
    }

    pub async fn start_receive_loop(self) -> Result<mpsc::Receiver<T>, StockServiceError> {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut read = self.read.lock().await;

            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        debug!("Received WebSocket text message: {}", text);
                        match serde_json::from_str::<T>(&text) {
                            Ok(parsed) => {
                                debug!("Successfully parsed WebSocket message: {:?}", parsed);
                                if tx.send(parsed).await.is_err() {
                                    error!("Failed to send parsed message through channel");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!(
                                    "Failed to parse WebSocket message: {}. Raw message: {}",
                                    e,
                                    text
                                );
                            }
                        }
                    }
                    Ok(Message::Binary(bin)) => {
                        debug!("Received WebSocket binary message: {:?}", bin);
                    }
                    Ok(Message::Ping(ping)) => {
                        debug!("Received WebSocket ping: {:?}", ping);
                    }
                    Ok(Message::Pong(pong)) => {
                        debug!("Received WebSocket pong: {:?}", pong);
                    }
                    Ok(Message::Close(close_frame)) => {
                        info!("WebSocket connection closed: {:?}", close_frame);
                        break;
                    }
                    Ok(Message::Frame(frame)) => {
                        debug!("Received WebSocket frame: {:?}", frame);
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}
