use std::{sync::Arc, time::Duration};

use bp_scheduler::actuator::Actuator;
use buttplug::core::message::ActuatorType;
use cxx::{CxxString, CxxVector};

use crate::{connection::Task, settings::TkDeviceSettings};

pub fn sanitize_name_list(list: &[String]) -> Vec<String> {
    list.iter()
        .map(|e| String::from(e.to_lowercase().trim()))
        .collect()
}

pub fn parse_list_string(input: &str) -> Vec<String> {
    let mut list = vec![];
    for part in input.split(',') {
        if !part.is_empty() {
            list.push(part.trim().to_lowercase());
        }
    }
    list
}

pub fn get_duration_from_secs(secs: f32) -> Duration {
    if secs > 0.0 {
        Duration::from_millis((secs * 1000.0) as u64)
    } else {
        Duration::MAX
    }
}

pub fn read_input_string(list: &CxxVector<CxxString>) -> Vec<String> {
    // automatically discards any empty strings to account for papyrus
    // inability to do dynamic array sizes
    list.iter()
        .filter(|d| !d.is_empty())
        .map(|d| d.to_string_lossy().into_owned())
        .collect()
}

#[derive(Clone, Debug)]
pub struct TkParams<'a> {
    pub selector: Vec<String>,
    pub task: &'a Task,
    pub events: Vec<String>,
}

impl<'a> TkParams<'a> {
    pub fn filter_devices(
        &self,
        actuators: &[Arc<Actuator>],
        actuator_types: &[ActuatorType],
    ) -> Vec<Arc<Actuator>> {
        actuators
            .iter()
            .filter(|a| {
                self.selector.iter().any(|x| x == a.identifier())
                    && a.device.message_attributes().scalar_cmd().is_some()
            })
            .filter(|a| actuator_types.iter().any(|x| x == &a.actuator))
            .cloned()
            .collect()
    }

    pub fn from_input(events: Vec<String>, task: &'a Task, devices: &[TkDeviceSettings]) -> Self {
        let event_names = sanitize_name_list(&events);
        let actuator_ids = devices
            .iter()
            .filter(|d| {
                d.enabled
                    && (event_names.is_empty() || d.events.iter().any(|e| event_names.contains(e)))
            })
            .map(|d| d.actuator_id.clone())
            .collect();
        TkParams {
            selector: actuator_ids,
            task,
            events,
        }
    }
}
