use rand::{thread_rng, Rng};
use rumble::api::{Central, Peripheral, UUID};
use rumble::bluez::manager::Manager;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub fn main() {
    let manager = Manager::new().unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().unwrap();
    let mut adapter = adapters.into_iter().nth(0).unwrap();

    // reset the adapter -- clears out any errant state
    adapter = manager.down(&adapter).unwrap();
    adapter = manager.up(&adapter).unwrap();

    // connect to the adapter
    let central = adapter.connect().unwrap();

    // start scanning for devices
    central.start_scan().unwrap();
    // instead of waiting, you can use central.on_event to be notified of
    // new devices
    thread::sleep(Duration::from_secs(5));

    // find the device we're interested in
    let light = central
        .peripherals()
        .into_iter()
        .find(|p| {
            p.properties()
                .local_name
                .iter()
                .any(|name| name.contains("bell"))
        })
        .unwrap();

    if !light.is_connected() {
        // connect to the device
        light.connect().unwrap();
    }

    // discover characteristics
    light.discover_characteristics().unwrap();

    // find the characteristic we want
    let chars = light.characteristics();
    let hid_char = chars
        .iter()
        .find(|c| {
            c.uuid
                == UUID::B128([
                    0x00, 0x00, 0x88, 0x5a, 0x00, 0x00, 0x10, 0x00, 0x80, 00, 0x00, 0x80, 0x5f,
                    0x9b, 0x34, 0xfb,
                ])
        })
        .unwrap();

    let event_handler = |msg| {
        println!("Recv: {:?}", msg);
    };

    light.on_notification(Box::new(event_handler));

    light.subscribe(hid_char).unwrap();
}
