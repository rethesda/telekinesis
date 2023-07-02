use buttplug::{
    client::ButtplugClient,
    core::{
        connector::{
            ButtplugConnector, ButtplugInProcessClientConnector,
            ButtplugInProcessClientConnectorBuilder,
        },
        message::{ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage},
    },
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};
use futures::{StreamExt, Future};
use std::fmt::{self};
use tokio::{runtime::Runtime, sync::mpsc::channel, sync::mpsc::unbounded_channel};
use tracing::{debug, error, info, warn};

use crate::{
    commands::{create_cmd_thread, TkAction},
    Speed, Tk, TkEvent,
};

pub struct Telekinesis {
    pub event_receiver: tokio::sync::mpsc::UnboundedReceiver<TkEvent>,
    pub command_sender: tokio::sync::mpsc::Sender<TkAction>,
    pub thread: Runtime,
}

pub fn in_process_connector() -> ButtplugInProcessClientConnector {
    ButtplugInProcessClientConnectorBuilder::default()
        .server(
            ButtplugServerBuilder::default()
                .comm_manager(BtlePlugCommunicationManagerBuilder::default())
                .finish()
                .expect("Could not create in-process-server."),
        )
        .finish()
}

impl Telekinesis {
    pub fn connect_with<T, Fn, Fut>(connector_factory: Fn) -> Result<Telekinesis, anyhow::Error>
    where
        Fn: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send,
        T: ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
            + 'static,
    {
        let (event_sender, event_receiver) = unbounded_channel();
        let (command_sender, command_receiver) = channel(256); // we handle them immediately
        let runtime = Runtime::new()?;
        runtime.spawn(async move {
            info!("Main thread started");
            let buttplug = with_connector(connector_factory().await).await;
            let mut events = buttplug.event_stream();
            create_cmd_thread(buttplug, event_sender.clone(), command_receiver);

            while let Some(event) = events.next().await {
                event_sender
                    .send(TkEvent::from_event(event))
                    .unwrap_or_else(|_| warn!("Dropped event cause queue is full."));
            }
        });
        Ok(Telekinesis {
            command_sender: command_sender,
            event_receiver: event_receiver,
            thread: runtime,
        })
    }
}

impl fmt::Debug for Telekinesis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Telekinesis").finish()
    }
}

impl Tk for Telekinesis {
    fn scan_for_devices(&self) -> bool {
        info!("Sending Command: Scan for devices");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkScan) {
            error!("Failed to send vibrate_all"); // whats skyrim gonna do about it
            return false;
        }
        true
    }

    fn vibrate_all(&self, speed: Speed) -> bool {
        info!("Sending Command: Vibrate all");
        if let Err(_) = self.command_sender.try_send(TkAction::TkVibrateAll(speed)) {
            error!("Failed to send vibrate_all");
            return false;
        }
        true
    }

    fn vibrate_all_delayed(&self, speed: Speed, duration: std::time::Duration) -> bool {
        info!("Sending Command: Vibrate all delayed");
        if let Err(_) = self
            .command_sender
            .try_send(TkAction::TkVibrateAllDelayed(speed, duration))
        {
            error!("Failed to send delayed command");
            return false;
        }
        true
    }

    fn stop_all(&self) -> bool {
        info!("Sending Command: Stop all");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkStopAll) {
            error!("Failed to send stop_all");
            return false;
        }
        true
    }

    fn disconnect(&mut self) {
        info!("Sending Command: Disconnecting client");
        if let Err(_) = self.command_sender.blocking_send(TkAction::TkDiscconect) {
            error!("Failed to send disconnect");
        }
    }

    fn get_next_event(&mut self) -> Option<TkEvent> {
        if let Ok(msg) = self.event_receiver.try_recv() {
            debug!("Got event {}", msg.to_string());
            return Some(msg);
        }
        None
    }

    fn get_next_events(&mut self) -> Vec<TkEvent> {
        let mut events = vec![];
        while let Some(event) = self.get_next_event() {
            events.push(event);
            if events.len() >= 128 {
                break;
            }
        }
        events
    }

    // pub fn tk_get_connected_devices(&self) {
    //     self.runtime.block_on(async {
    //         self.client
    //                .devices()
    //                .iter()
    //                .filter( |f| self.is_vibrator(&f))
    //                .map(|f| f.name().clone() )
    //                .collect::<Vec<String>>()
    //     });
    // }
}

async fn with_connector<T>(connector: T) -> ButtplugClient
where
    T: ButtplugConnector<ButtplugCurrentSpecClientMessage, ButtplugCurrentSpecServerMessage>
        + 'static,
{
    let buttplug = ButtplugClient::new("Telekinesis");
    let bp =  buttplug
        .connect(connector)
        .await;
    match bp {
        Ok(_) => {
            info!("Connected client.")
        },
        Err(err) => {
            error!("Could not connect client. Error: {}.", err);
        },
    }
    buttplug
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration, vec};

    use buttplug::core::message::ActuatorType;
    use tracing::Level;

    use crate::fakes::{FakeDeviceConnector, scalar};

    use super::*;
    
    #[allow(dead_code)]
    fn enable_log() {
        tracing::subscriber::set_global_default(
            tracing_subscriber::fmt()
                .with_max_level(Level::INFO)
                .finish(),
        )
        .unwrap();
    }
    
    #[test]
    fn test_regular_connection() {
        // arrange
        let mut tk = Telekinesis::connect_with(|| async move { in_process_connector() }).unwrap();
        tk.stop_all();
        thread::sleep(Duration::from_millis(200));
        assert!(tk.get_next_event().is_some())
    }

    #[test]
    fn test_demo_vibrate_only_vibrators() {
        // arrange
        enable_log();
        let (connector, call_registry) = FakeDeviceConnector::device_demo();

        // act
        let tk = Telekinesis::connect_with(|| async move { connector }).unwrap();
        
        tk.vibrate_all(Speed::new(100));
        thread::sleep(Duration::from_millis(500));

        // assert
        assert_eq!(call_registry.get_record(1).len(), 1);
        assert_eq!(call_registry.get_record(4).len(), 0);
    }

    #[test]
    fn test_demo_vibrate_only_vibrates_actuator_vibrate() {
        // arrange
        let (connector, call_registry) = FakeDeviceConnector::new( vec![
            scalar(1, "vib1", ActuatorType::Vibrate),
            scalar(2, "vib2", ActuatorType::Inflate)
        ]);

        // act
        let tk = Telekinesis::connect_with(|| async move { connector }).unwrap();
        tk.vibrate_all(Speed::new(100));
        thread::sleep(Duration::from_millis(500));

        // assert
        assert_eq!(call_registry.get_record(1).len(), 1);
        assert_eq!(call_registry.get_record(2).len(), 0);
    }
}
