//! BLE link to the wheel: scan, connect, and a strictly serialized request/reply engine.
//!
//! The firmware handles one request at a time and answers with `request_id | 0x80`.
//! Unsupported commands get no reply, and missing profile pages answer with 20 zero bytes.

use std::sync::Arc;
use std::time::Duration;

use btleplug::api::{
    Central, CentralEvent, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

use crate::protocol::{self, hex};

const WRITE_UUID: Uuid = Uuid::from_u128(0x0000ffe1_0000_1000_8000_00805f9b34fb);
const NOTIFY_UUID: Uuid = Uuid::from_u128(0x0000ffe2_0000_1000_8000_00805f9b34fb);
const NAME_PREFIX: &str = "FFB WHEEL";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundDevice {
    pub id: String,
    pub name: String,
    pub rssi: Option<i16>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub dir: &'static str,
    pub hex: String,
    pub note: Option<String>,
}

struct Pending {
    expect: u8,
    tx: oneshot::Sender<Vec<u8>>,
}

struct Connected {
    peripheral: Peripheral,
    write: Characteristic,
    name: String,
}

pub struct Link {
    app: AppHandle,
    adapter: Mutex<Option<Adapter>>,
    conn: Mutex<Option<Connected>>,
    pending: Arc<std::sync::Mutex<Option<Pending>>>,
    /// Held for the whole send→reply cycle so requests never overlap.
    turn: Mutex<()>,
}

impl Link {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            adapter: Mutex::new(None),
            conn: Mutex::new(None),
            pending: Arc::new(std::sync::Mutex::new(None)),
            turn: Mutex::new(()),
        }
    }

    async fn adapter(&self) -> Result<Adapter, String> {
        let mut guard = self.adapter.lock().await;
        if let Some(a) = guard.as_ref() {
            return Ok(a.clone());
        }
        let manager = Manager::new().await.map_err(|e| format!("Bluetooth unavailable: {e}"))?;
        let adapter = manager
            .adapters()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .next()
            .ok_or("No Bluetooth adapter found")?;
        *guard = Some(adapter.clone());
        Ok(adapter)
    }

    pub async fn scan(&self, millis: u64) -> Result<Vec<FoundDevice>, String> {
        let adapter = self.adapter().await?;
        adapter.start_scan(ScanFilter::default()).await.map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_millis(millis)).await;
        let _ = adapter.stop_scan().await;
        let mut found = Vec::new();
        for p in adapter.peripherals().await.map_err(|e| e.to_string())? {
            let Ok(Some(props)) = p.properties().await else { continue };
            let Some(name) = props.local_name else { continue };
            if name.to_uppercase().starts_with(NAME_PREFIX) {
                found.push(FoundDevice { id: p.id().to_string(), name, rssi: props.rssi });
            }
        }
        found.sort_by_key(|d| std::cmp::Reverse(d.rssi.unwrap_or(i16::MIN)));
        Ok(found)
    }

    pub async fn connect(&self, id: &str) -> Result<String, String> {
        self.disconnect().await;
        let adapter = self.adapter().await?;
        let mut target = None;
        for p in adapter.peripherals().await.map_err(|e| e.to_string())? {
            if p.id().to_string() == id {
                target = Some(p);
                break;
            }
        }
        let p = target.ok_or("Wheel not found — scan again")?;
        let name = p
            .properties()
            .await
            .ok()
            .flatten()
            .and_then(|x| x.local_name)
            .unwrap_or_else(|| NAME_PREFIX.into());

        tokio::time::timeout(Duration::from_secs(15), p.connect())
            .await
            .map_err(|_| "Connection timed out")?
            .map_err(|e| format!("Could not connect: {e}"))?;
        p.discover_services().await.map_err(|e| e.to_string())?;
        let chars = p.characteristics();
        let write = chars.iter().find(|c| c.uuid == WRITE_UUID).cloned().ok_or(
            "This device does not expose the LiteStar configuration service (ffe0/ffe1)",
        )?;
        let notify = chars
            .iter()
            .find(|c| c.uuid == NOTIFY_UUID)
            .cloned()
            .ok_or("Notification characteristic ffe2 missing")?;
        p.subscribe(&notify).await.map_err(|e| e.to_string())?;

        let mut stream = p.notifications().await.map_err(|e| e.to_string())?;
        let pending = self.pending.clone();
        let app = self.app.clone();
        tokio::spawn(async move {
            while let Some(n) = stream.next().await {
                if n.uuid != NOTIFY_UUID {
                    continue;
                }
                let d = n.value;
                if protocol::is_live_frame(&d) {
                    let _ = app.emit("live", protocol::decode_live(&d));
                    continue;
                }
                let _ = app.emit("log", LogEntry { dir: "rx", hex: hex(&d), note: None });
                let mut slot = pending.lock().unwrap();
                let matches = slot
                    .as_ref()
                    .is_some_and(|p| d.first() == Some(&p.expect) || d.iter().all(|b| *b == 0));
                if matches {
                    let _ = slot.take().unwrap().tx.send(d);
                }
            }
            let _ = app.emit("connection", "disconnected");
        });

        // Watch for the link dropping.
        let app = self.app.clone();
        let pid = p.id();
        if let Ok(mut events) = adapter.events().await {
            tokio::spawn(async move {
                while let Some(ev) = events.next().await {
                    if let CentralEvent::DeviceDisconnected(id) = ev {
                        if id == pid {
                            let _ = app.emit("connection", "disconnected");
                            break;
                        }
                    }
                }
            });
        }

        *self.conn.lock().await = Some(Connected { peripheral: p, write, name: name.clone() });
        Ok(name)
    }

    pub async fn disconnect(&self) {
        if let Some(c) = self.conn.lock().await.take() {
            let _ = c.peripheral.disconnect().await;
        }
    }

    pub async fn connected_name(&self) -> Option<String> {
        self.conn.lock().await.as_ref().map(|c| c.name.clone())
    }

    /// Sends `data` and waits for its reply. Returns `Ok(None)` if the device stayed silent
    /// after all attempts — which for this firmware means "unsupported".
    pub async fn request(&self, data: &[u8], attempts: u32) -> Result<Option<Vec<u8>>, String> {
        let _turn = self.turn.lock().await;
        let expect = protocol::response_id(data[0]);
        for attempt in 0..attempts.max(1) {
            let (tx, rx) = oneshot::channel();
            *self.pending.lock().unwrap() = Some(Pending { expect, tx });
            {
                let conn = self.conn.lock().await;
                let c = conn.as_ref().ok_or("Not connected")?;
                let _ = self.app.emit(
                    "log",
                    LogEntry {
                        dir: "tx",
                        hex: hex(data),
                        note: (attempt > 0).then(|| format!("retry {attempt}")),
                    },
                );
                c.peripheral
                    .write(&c.write, data, WriteType::WithoutResponse)
                    .await
                    .map_err(|e| format!("Write failed: {e}"))?;
            }
            match tokio::time::timeout(Duration::from_millis(700), rx).await {
                Ok(Ok(reply)) => {
                    // The vendor app paces requests 100 ms apart; the firmware likes it.
                    tokio::time::sleep(Duration::from_millis(60)).await;
                    return Ok(Some(reply));
                }
                _ => {
                    self.pending.lock().unwrap().take();
                }
            }
        }
        let _ = self.app.emit(
            "log",
            LogEntry { dir: "rx", hex: String::new(), note: Some("no reply".into()) },
        );
        Ok(None)
    }

    /// Fire-and-forget write, used for test-mode exit where the reply doesn't matter.
    pub async fn send(&self, data: &[u8]) -> Result<(), String> {
        let _turn = self.turn.lock().await;
        let conn = self.conn.lock().await;
        let c = conn.as_ref().ok_or("Not connected")?;
        let _ = self.app.emit("log", LogEntry { dir: "tx", hex: hex(data), note: None });
        c.peripheral
            .write(&c.write, data, WriteType::WithoutResponse)
            .await
            .map_err(|e| format!("Write failed: {e}"))
    }
}
