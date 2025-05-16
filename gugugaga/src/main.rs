use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use eyre::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tracing::{error, info, instrument};
use zbus::{
    zvariant::{Type, Value},
    Connection, MessageStream,
};

#[derive(Parser, Debug)]
#[clap(version, about = "DBus notification to danmaku server forwarder")]
struct Args {
    /// WebSocket服务器地址
    #[clap(long, env = "WS_SERVER")]
    ws_server: String,

    /// 要监听的应用名称
    #[clap(long, env = "APP_NAME", default_value = "QQ")]
    app_name: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
struct Notification<'a> {
    app_name: &'a str,
    id: u32,
    icon: &'a str,
    summary: Arc<str>,
    body: Arc<str>,
    actions: Vec<Arc<str>>,
    hints: HashMap<Arc<str>, Value<'a>>,
    timeout: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Danmaku {
    pub text: SmolStr,
    pub color: Option<Arc<str>>,
    pub size: Option<f64>,
    pub sender: Option<SmolStr>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DanmakuPacket {
    pub group: Arc<str>,
    pub danmaku: Danmaku,
}

#[instrument]
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 解析命令行参数和环境变量
    let args = Args::parse();
    let ws_server = args.ws_server;
    let app_name = args.app_name;

    info!(
        ws_server = %ws_server,
        app_name = %app_name,
        "DBus notification forwarding service started"
    );

    // Create channel for DBus <-> WebSocket communication
    let (tx, rx) = mpsc::channel::<DanmakuPacket>(32);

    // Wait for both tasks to complete
    tokio::try_join!(
        listen_dbus_notifications(tx, &app_name),
        websocket_client(rx, &ws_server)
    )?;

    info!("Service exited normally");
    Ok(())
}

#[instrument(skip(tx))]
async fn listen_dbus_notifications(tx: mpsc::Sender<DanmakuPacket>, app_name: &str) -> Result<()> {
    let connection = Connection::session().await?;

    // Define match rules for monitoring messages
    let match_rules = vec![
        // Monitor all notifications
        "type='method_call',interface='org.freedesktop.Notifications',member='Notify'",
    ];

    // Call BecomeMonitor method
    connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus.Monitoring"),
            "BecomeMonitor",
            &(match_rules, 0u32),
        )
        .await?;
    info!("Registered as DBus monitor");

    // Create message stream
    let mut stream = MessageStream::from(connection);
    while let Some(msg) = stream.next().await {
        let msg = msg?;

        // Parse notification parameters
        let body = msg.body();
        let Ok(notification) = body.deserialize::<Notification>() else {
            info!(
                signature = body.signature().to_string(),
                "Unknown notification format"
            );
            continue;
        };

        info!(
            app_name = notification.app_name,
            notification_id = notification.id,
            summary = notification.summary.as_ref(),
            "Received new notification"
        );

        if notification.app_name != app_name {
            continue;
        }
        let Some((sender, text)) = notification.body.split_once('：') else {
            continue;
        };
        let group = &notification.summary;

        let danmaku = Danmaku {
            text: text.into(),
            color: None,
            size: None,
            sender: Some(sender.into()),
        };
        let packet = DanmakuPacket {
            group: group.clone(),
            danmaku,
        };

        if let Err(e) = tx.send(packet).await {
            error!(error = %e, "Failed to send notification to channel");
            break;
        }
    }

    Ok(())
}

#[instrument(skip(rx))]
async fn websocket_client(mut rx: mpsc::Receiver<DanmakuPacket>, ws_server: &str) -> Result<()> {
    // Initial connection
    let (mut ws_stream, _) = connect_async(ws_server).await?;
    info!(ws_server = %ws_server, "Connected to WebSocket server");

    while let Some(packet) = rx.recv().await {
        let json = serde_json::to_string(&packet)?;

        // Try to send notification
        match ws_stream.send(json.into()).await {
            Ok(_) => info!(
                group = packet.group.as_ref(),
                sender = packet.danmaku.sender.as_deref(),
                text = packet.danmaku.text.as_str(),
                "Notification sent to WebSocket server"
            ),
            Err(e) => {
                error!(error = %e, "Send failed, attempting to reconnect");

                // Reconnect
                match connect_async(ws_server).await {
                    Ok((new_stream, _)) => {
                        ws_stream = new_stream;
                        info!("Reconnected to WebSocket server");

                        // Resend current notification
                        if let Err(e) = ws_stream.send(serde_json::to_string(&packet)?.into()).await
                        {
                            error!(error = %e, "Resend failed");
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Reconnection failed");
                    }
                }
            }
        }
    }

    Ok(())
}
