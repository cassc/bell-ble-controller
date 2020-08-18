#[allow(unused_imports)]
use btleplug::api::{Central, CentralEvent, Characteristic, Peripheral, UUID};
#[allow(unused_imports)]
#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::Adapter, adapter::ConnectedAdapter, manager::Manager};
#[allow(unused_imports)]
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};
#[allow(unused_imports)]
#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};
#[allow(unused_imports)]
use rand::{thread_rng, Rng};
use std::str::FromStr;
#[allow(dead_code)]
#[allow(unused_imports)]
use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
fn connect_to(adapter: &Adapter) -> ConnectedAdapter {
    adapter
        .connect()
        .expect("Error connecting to BLE Adapter....") //linux
}
#[cfg(target_os = "linux")]
fn print_adapter_info(adapter: &ConnectedAdapter) {
    println!(
        "connected adapter {:?} is UP: {:?}",
        adapter.adapter.name,
        adapter.adapter.is_up()
    );
    println!("adapter states : {:?}", adapter.adapter.states);
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn connect_to(adapter: &Adapter) -> &Adapter {
    adapter //windows 10
}
#[cfg(any(target_os = "windows", target_os = "macos"))]
fn print_adapter_info(_adapter: &Adapter) {
    println!("adapter info can't be printed on Windows 10 or mac");
}

#[cfg(target_os = "linux")]
fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

fn conn_bell_in_peripherals(central: &ConnectedAdapter) -> impl Peripheral {
    // all peripheral devices in range
    loop {
        for peripheral in central.peripherals().iter() {
            println!(
                "peripheral : {:?} is connected: {:?}",
                peripheral.properties().local_name,
                peripheral.is_connected()
            );

            if let Some(name) = peripheral.properties().local_name {
                println!("Found device {} {:?}", name, peripheral.address());
                if name.contains("bell") {
                    println!("found bell controller {:?}", name);
                    if !peripheral.is_connected() {
                        let r = peripheral.connect();
                        if r.is_err() {
                            println!("Connect to {:?} failed {:?}", peripheral, r);
                            return peripheral.clone();
                        } else {
                            println!("Connect to {:?} success", peripheral);
                            return peripheral.clone();
                        }
                    } else {
                        return peripheral.clone();
                    }
                }
            } else {
                println!("found device with no name {:?}", peripheral);
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn main() {
    let manager = Manager::new().unwrap();
    let adapter_list = manager.adapters().unwrap();
    if adapter_list.len() <= 0 {
        eprintln!("Bluetooth adapter(s) were NOT found, sorry...\n");
    } else {
        let manager = Manager::new().unwrap();

        let central = get_central(&manager);

        let event_receiver = central.event_receiver().unwrap();

        // start scanning for devices
        central.start_scan().unwrap();

        while let Ok(event) = event_receiver.recv() {
            println!("Event: {:?}", event);
            match event {
                CentralEvent::DeviceConnected(bd_addr) => {
                    break;
                }
                _ => {}
            }
        }

        let controller = loop {
            let device = central.peripherals().into_iter().find(|p| {
                p.properties()
                    .local_name
                    .iter()
                    .any(|name| name.contains("bell"))
            });
            if device.is_some() {
                break device.unwrap().clone();
            } else {
                println!("no device found, wait");
                thread::sleep(Duration::from_secs(1));
            }
        };

        central.stop_scan().unwrap();

        while !controller.is_connected() {
            let r = controller.connect();
            println!("conn returns {:?}", r);
            thread::sleep(Duration::from_secs(5));
        }

        println!("device connnected");

        let handler = |msg| {
            println!("event: {:?}", msg);
        };
        controller.on_notification(Box::new(handler));

        let chars = controller.characteristics();
        for ch in &chars {
            println!("chr: {:?}", ch);
        }
        let cmd_char = chars
            .iter()
            .find(|c| c.uuid == UUID::from_str("0000885a-0000-1000-8000-00805f9b34fb").unwrap())
            .unwrap();

        let cmd = vec![0x01, 0x00];
        controller.command(&cmd_char, &cmd).unwrap();
        loop {
            let r = controller.read(cmd_char);
            println!("Ble msg: {:?}", r);
        }
    }
}
