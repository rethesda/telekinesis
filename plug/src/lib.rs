use anyhow::Error;
use buttplug::client::ButtplugClientDevice;
use connection::{TkAction, TkConnectionEvent, TkConnectionStatus, TkStatus};
use lazy_static::lazy_static;
use pattern::{get_pattern_names, TkButtplugScheduler};
use settings::PATTERN_PATH;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Sender, UnboundedReceiver, UnboundedSender},
};
use tracing::{error, info, instrument};

use cxx::{CxxString, CxxVector};
use telekinesis::ERROR_HANDLE;

use crate::{
    input::read_input_string,
    settings::{TkSettings, SETTINGS_FILE, SETTINGS_PATH},
};

mod connection;
mod fakes;
mod input;
mod logging;
mod pattern;
mod settings;
mod telekinesis;
mod tests;
mod util;

/// The ffi interfaces called as papyrus native functions. This is very thin glue code to
/// store the global singleton state in a mutex and handle error conditions, and then
/// acess the functionality in the main Telekinesis struct
///
/// - All ffi methods are non-blocking, triggering an async action somewhere in the future
/// - All all error conditions during the function call (i.e. mutex not available) will
///   be swallowed and logged to Telekinesis.log
#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn tk_connect() -> bool;
        fn tk_scan_for_devices() -> bool;
        fn tk_stop_scan() -> bool;
        
        fn tk_get(key: &str) -> String;
        fn tk_get_devices() -> Vec<String>; // GetValues("devices")   Query                 -> String[]
        fn tk_get_device_connected(device_name: &str) -> bool; // GetValue ("device.status.WeVibe Bond")       -> String
        fn tk_get_device_capabilities(device_name: &str) -> Vec<String>; // GetValues("device.capabilities.WeVibe Bond") -> String[]
        fn tk_get_pattern_names(vibration_devices: bool) -> Vec<String>; // GetValues("patterns")                        -> String[]

        fn tk_vibrate(speed: i64, secs: f32, events: &CxxVector<CxxString>) -> i32;
        fn tk_vibrate_pattern(pattern_name: &str, secs: f32, events: &CxxVector<CxxString>) -> i32;
        fn tk_stop(handle: i32) -> bool;
        fn tk_stop_all() -> bool;
        fn tk_close() -> bool;
        fn tk_process_events() -> Vec<String>;
        fn tk_settings_set(key: &str, value: &str) -> bool; // SetText  ( "devices.settings", "" ) -> Bool
        fn tk_settings_set_enabled(device_name: &str, enabled: bool); // SetOption( "WeVibeBond", "devices.enabled", true/false)
        fn tk_settings_get_enabled(device_name: &str) -> bool; // GetOption( , "devices.enabled.WeVibe Bond")
        fn tk_settings_get_events(device_name: &str) -> Vec<String>; // SetOption
        fn tk_settings_set_events(device_name: &str, events: &CxxVector<CxxString>); //
        fn tk_settings_store() -> bool;
    }
}

type DeviceList = Vec<Arc<ButtplugClientDevice>>;

/// access to Telekinesis struct from within foreign rust modules and tests
pub trait Tk {
    fn connect(settings: TkSettings) -> Result<Telekinesis, Error>;
    fn scan_for_devices(&self) -> bool;
    fn stop_scan(&self) -> bool;
    fn disconnect(&mut self);
    fn get_connection_status(&self) -> Option<TkConnectionStatus>;
    fn get_devices(&self) -> DeviceList;
    fn get_device_names(&self) -> Vec<String>;
    fn get_device_connected(&self, device_name: &str) -> bool;
    fn get_device_capabilities(&self, device_name: &str) -> Vec<String>;
    fn vibrate(&mut self, speed: Speed, duration: TkDuration, events: Vec<String>) -> i32;
    fn vibrate_pattern(&mut self, pattern: TkPattern, events: Vec<String>) -> i32;
    fn stop(&mut self, handle: i32) -> bool;
    fn stop_all(&mut self) -> bool;
    fn get_next_event(&mut self) -> Option<TkConnectionEvent>;
    fn process_next_events(&mut self) -> Vec<TkConnectionEvent>;
    fn settings_set_enabled(&mut self, device_name: &str, enabled: bool);
    fn settings_set_events(&mut self, device_name: &str, events: Vec<String>);
    fn settings_get_events(&self, device_name: &str) -> Vec<String>;
    fn settings_get_enabled(&self, device_name: &str) -> bool;
}

pub struct Telekinesis {
    pub connection_status: Arc<Mutex<TkStatus>>,
    settings: TkSettings,
    runtime: Runtime,
    command_sender: Sender<TkAction>,
    scheduler: TkButtplugScheduler,
    connection_events: UnboundedReceiver<TkConnectionEvent>,
    event_sender: UnboundedSender<TkConnectionEvent>
}

#[derive(Debug, Clone, Copy)]
pub struct Speed {
    pub value: u16,
}

#[derive(Clone, Debug)]
pub enum TkDuration {
    Infinite,
    Timed(Duration),
}

impl TkDuration {
    pub fn from_input_float(secs: f32) -> TkDuration {
        if secs > 0.0 {
            return TkDuration::Timed(Duration::from_millis((secs * 1000.0) as u64));
        } else {
            return TkDuration::Infinite;
        }
    }
    pub fn from_millis(ms: u64) -> TkDuration {
        TkDuration::Timed(Duration::from_millis(ms))
    }
    pub fn from_secs(s: u64) -> TkDuration {
        TkDuration::Timed(Duration::from_secs(s))
    }
}

#[derive(Clone, Debug)]
pub enum TkPattern {
    Linear(TkDuration, Speed),
    Funscript(TkDuration, String),
}

lazy_static! {
    static ref TK: Mutex<Option<Telekinesis>> = Mutex::new(None);
}

fn access_mutex<F, R>(func: F) -> Option<R>
where
    F: FnOnce(&mut Telekinesis) -> R,
{
    if let Ok(mut guard) = TK.try_lock() {
        match guard.take() {
            Some(mut tk) => {
                let result = Some(func(&mut tk));
                guard.replace(tk);
                return result;
            }
            None => error!("Trying to call method on non-initialized tk"),
        }
    }
    None
}

#[instrument]
pub fn tk_connect() -> bool {
    tk_connect_with_settings(TkSettings::try_read_or_default(
        SETTINGS_PATH,
        SETTINGS_FILE,
    ))
}

#[instrument]
pub fn tk_connect_with_settings(settings: TkSettings) -> bool {
    match Telekinesis::connect(settings) {
        Ok(tk) => {
            match TK.try_lock() {
                Ok(mut guard) => {
                    guard.replace(tk);
                }
                Err(err) => error!("Failed locking mutex: {}", err),
            }
            true
        }
        Err(err) => {
            error!("tk_connect error {:?}", err);
            false
        }
    }
}

#[instrument]
pub fn tk_close() -> bool {
    info!("Closing connection");
    match TK.try_lock() {
        Ok(mut guard) => {
            if let Some(mut tk) = guard.take() {
                tk.disconnect();
                return true;
            }
        }
        Err(err) => error!("Failed locking mutex: {}", err),
    }
    false
}

#[instrument]
pub fn tk_scan_for_devices() -> bool {
    access_mutex(|tk| tk.scan_for_devices()).is_some()
}

#[instrument]
pub fn tk_stop_scan() -> bool {
    access_mutex(|tk| tk.stop_scan()).is_some()
}

#[instrument]
pub fn tk_vibrate(speed: i64, secs: f32, events: &CxxVector<CxxString>) -> i32 {
    access_mutex(|tk| {
        tk.vibrate(
            Speed::new(speed),
            TkDuration::from_input_float(secs),
            read_input_string(&events),
        )
    })
    .unwrap_or(ERROR_HANDLE)
}

#[instrument]
pub fn tk_vibrate_pattern(pattern_name: &str, secs: f32, events: &CxxVector<CxxString>) -> i32 {
    access_mutex(|tk| {
        tk.vibrate_pattern(
            TkPattern::Funscript(
                TkDuration::from_input_float(secs),
                String::from(pattern_name),
            ),
            read_input_string(&events),
        )
    })
    .unwrap_or(ERROR_HANDLE)
}

#[instrument]
pub fn tk_stop(handle: i32) -> bool {
    access_mutex(|tk| tk.stop(handle)).is_some()
}

#[instrument]
pub fn tk_get_devices() -> Vec<String> {
    if let Some(value) = access_mutex(|tk| tk.get_device_names()) {
        return value;
    }
    vec![]
}

#[instrument]
pub fn tk_get(key: &str) -> String {
    if let Some(value) = access_mutex(|tk| match key {
        "connection.status" => tk
            .get_connection_status()
            .or(Some(TkConnectionStatus::NotConnected))
            .unwrap()
            .serialize_papyrus(),
        _ => {
            error!("Unknown key {}", key);
            String::from("")
        }
    }) {
        return value;
    }
    String::from("")
}

#[instrument]
pub fn tk_get_device_connected(name: &str) -> bool {
    if let Some(value) = access_mutex(|tk| tk.get_device_connected(name)) {
        return value;
    }
    false
}

#[instrument]
pub fn tk_get_device_capabilities(name: &str) -> Vec<String> {
    // 3
    if let Some(value) = access_mutex(|tk| tk.get_device_capabilities(name)) {
        return value;
    }
    vec![]
}

#[instrument]
pub fn tk_get_pattern_names(vibration_patterns: bool) -> Vec<String> {
    // 2
    get_pattern_names(PATTERN_PATH, vibration_patterns)
}

#[instrument]
pub fn tk_stop_all() -> bool {
    access_mutex(|tk| tk.stop_all()).is_some()
}

#[instrument]
pub fn tk_process_events() -> Vec<String> {
    match access_mutex(|tk| {
        let events = tk
            .process_next_events()
            .iter()
            .map(|evt| evt.serialize_papyrus())
            .collect::<Vec<String>>();
        return events;
    }) {
        Some(events) => events,
        None => vec![],
    }
}

#[instrument]
pub fn tk_settings_set(key: &str, value: &str) -> bool {
    info!("setting");
    access_mutex(|tk| {
        let mut settings = tk.settings.clone();
        let success = settings.set_string(key, value);
        tk.settings = settings;
        success
    })
    .is_some()
}

#[instrument]
pub fn tk_settings_set_enabled(device_name: &str, enabled: bool) {
    access_mutex(|tk| tk.settings_set_enabled(device_name, enabled));
}

#[instrument]
pub fn tk_settings_get_events(device_name: &str) -> Vec<String> {
    // 1
    match access_mutex(|tk| tk.settings_get_events(device_name)) {
        Some(events) => events,
        None => vec![],
    }
}

#[instrument]
pub fn tk_settings_set_events(device_name: &str, events: &CxxVector<CxxString>) {
    access_mutex(|tk| tk.settings_set_events(device_name, read_input_string(events)));
}

#[instrument]
pub fn tk_settings_get_enabled(device_name: &str) -> bool {
    match access_mutex(|tk| tk.settings_get_enabled(device_name)) {
        Some(enabled) => enabled,
        None => false,
    }
}

#[instrument]
pub fn tk_settings_store() -> bool {
    access_mutex(|tk| tk.settings.try_write(SETTINGS_PATH, SETTINGS_FILE)).is_some()
}
