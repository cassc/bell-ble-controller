/// Run this command to turn on bluetooth first:
/// bluetoothctl power on

static BELL_CONTROLLER_SERVICE_UUID: &'static str = "00008850-0000-1000-8000-00805f9b34fb";
static BELL_CONTROLLER_CHARACTER_UUID: &'static str = "0000885a-0000-1000-8000-00805f9b34fb";
use blurz::bluetooth_adapter::BluetoothAdapter;
use blurz::bluetooth_device::BluetoothDevice;
use blurz::bluetooth_discovery_session::BluetoothDiscoverySession;
use blurz::bluetooth_event::BluetoothEvent;
use blurz::bluetooth_event::BluetoothEvent::RSSI;
use blurz::bluetooth_gatt_characteristic::BluetoothGATTCharacteristic;
use blurz::bluetooth_gatt_descriptor::BluetoothGATTDescriptor;
use blurz::bluetooth_gatt_service::BluetoothGATTService;
use blurz::bluetooth_session::BluetoothSession;
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::str;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

const UUID_REGEX: &str = r"([0-9a-f]{4})([0-9a-f]{4})-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}";

lazy_static! {
    static ref RE: Regex = Regex::new(UUID_REGEX).unwrap();
}

/// List characteristics in service
pub fn list_characteritics(service: &BluetoothGATTService, session: &BluetoothSession) {
    // list characteristics
    let characteristics = service.get_gatt_characteristics().unwrap();
    for characteristic_path in characteristics {
        let characteristic = BluetoothGATTCharacteristic::new(session, characteristic_path);
        let uuid = characteristic.get_uuid().unwrap();
        let assigned_number = RE
            .captures(&uuid)
            .unwrap()
            .get(2)
            .map_or("", |m| m.as_str());
        let flags = characteristic.get_flags().unwrap();

        // println!("Characteristic: {:?}", characteristic);
        println!(
            " Characteristic UUID: {}, Assigned Number: 0x{:?} Flags: {:?}",
            uuid, assigned_number, flags
        );

        list_descriptors(&characteristic, session);
    }
}

pub fn get_service<'a, 'b>(
    short_service_uuid: &'b str,
    device: &'a BluetoothDevice,
    session: &'a BluetoothSession,
) -> Option<BluetoothGATTService<'a>> {
    let services_list = device.get_gatt_services().unwrap();

    for service_path in services_list {
        let service = BluetoothGATTService::new(session, service_path.to_string());
        let uuid = service.get_uuid().unwrap();
        let assigned_number = RE
            .captures(&uuid)
            .unwrap()
            .get(2)
            .map_or("", |m| m.as_str());

        println!(
            "Service UUID: {:?} Assigned Number: 0x{:?}",
            uuid, assigned_number
        );
        if assigned_number == short_service_uuid {
            return Some(service.clone());
        }
    }
    None
}

pub fn get_characteritic<'a, 'b>(
    char_short_uuid: &'b str,
    service: &'a BluetoothGATTService,
    session: &'a BluetoothSession,
) -> Option<BluetoothGATTCharacteristic<'a>> {
    // list characteristics
    let characteristics = service.get_gatt_characteristics().unwrap();
    for characteristic_path in characteristics {
        let characteristic = BluetoothGATTCharacteristic::new(session, characteristic_path);
        let uuid = characteristic.get_uuid().unwrap();
        let assigned_number = RE
            .captures(&uuid)
            .unwrap()
            .get(2)
            .map_or("", |m| m.as_str());
        let flags = characteristic.get_flags().unwrap();

        // println!("Characteristic: {:?}", characteristic);
        println!(
            " Characteristic Assigned Number: 0x{:?} Flags: {:?}",
            assigned_number, flags
        );

        if assigned_number == char_short_uuid {
            return Some(characteristic.clone());
        }
    }
    return None;
}

/// List descriptors in characteristic
pub fn list_descriptors(characteristic: &BluetoothGATTCharacteristic, session: &BluetoothSession) {
    let descriptors = characteristic.get_gatt_descriptors().unwrap();
    for descriptor_path in descriptors {
        let descriptor = BluetoothGATTDescriptor::new(session, descriptor_path);
        let uuid = descriptor.get_uuid().unwrap();
        let assigned_number = RE
            .captures(&uuid)
            .unwrap()
            .get(2)
            .map_or("", |m| m.as_str());
        let value = descriptor.read_value(None).unwrap();
        let value = match &assigned_number[4..] {
            "2901" => str::from_utf8(&value).unwrap().to_string(),
            _ => format!("{:?}", value),
        };

        println!(
            "    Descriptor UUID: {}, Assigned Number: 0x{:?} Read Value: {:?}",
            uuid, assigned_number, value
        );
    }
}

pub fn explore_device(device: &BluetoothDevice, session: &BluetoothSession) {
    // list services
    let services_list = device.get_gatt_services().unwrap();

    for service_path in services_list {
        let service = BluetoothGATTService::new(session, service_path.to_string());
        let uuid = service.get_uuid().unwrap();
        let assigned_number = RE
            .captures(&uuid)
            .unwrap()
            .get(2)
            .map_or("", |m| m.as_str());

        println!(
            "Service UUID: {:?} Assigned Number: 0x{:?}",
            uuid, assigned_number
        );

        list_characteritics(&service, session);
        println!("");
    }
}

fn get_joysticks_paired(
    bt_session: &BluetoothSession,
) -> Result<Vec<BluetoothDevice>, Box<dyn Error>> {
    let adapter: BluetoothAdapter = BluetoothAdapter::init(bt_session)?;

    let mut devices = vec![];
    let device_list = adapter.get_device_list()?;

    for device_path in device_list {
        let device = BluetoothDevice::new(bt_session, device_path.to_string());
        println!(
            "Device: {:?} Name: {:?}, rssi: {:?}",
            device_path,
            device.get_name().ok(),
            device.get_rssi().ok()
        );
        if let Ok(name) = device.get_name() {
            if name.contains("bell") {
                devices.push(device);
            }
        }
    }

    Ok(devices)
}

fn get_joysticks_with_event(
    bt_session: &BluetoothSession,
    timeout_secs: u64,
) -> Result<Vec<BluetoothDevice>, Box<dyn Error>> {
    let adapter: BluetoothAdapter = BluetoothAdapter::init(bt_session)?;
    let adapter_id = adapter.get_id();
    let discover_session = BluetoothDiscoverySession::create_session(&bt_session, adapter_id)?;

    let start = SystemTime::now();
    let mut devices = vec![];
    discover_session.start_discovery()?;

    for event in bt_session.incoming(1000).map(BluetoothEvent::from) {
        let now = SystemTime::now();
        if now > start + Duration::from_secs(timeout_secs) {
            println!("discovery timeout");
            break;
        }
        match event {
            Some(RSSI { object_path, rssi }) => {
                let device = BluetoothDevice::new(bt_session, object_path.clone());

                if let Ok(name) = device.get_name() {
                    println!("{} {} {}", &object_path, rssi, name);
                    if name.contains("bell") {
                        devices.push(device.clone())
                    }
                } else {
                    println!("{} {}", &object_path, rssi);
                }
            }
            _ => println!("{:?}", event),
        }
    }

    discover_session.stop_discovery()?;
    Ok(devices)
}

fn enable_joystick_notify(
    bt_session: &BluetoothSession,
    device: &BluetoothDevice,
) -> Result<(), Box<dyn Error>> {
    let uuid_service = "8850";
    let uuid_characteritic = "885a";

    if let Some(service) = get_service(uuid_service, &device, bt_session) {
        let session = bt_session;

        if let Some(ch) = get_characteritic(uuid_characteritic, &service, session) {
            ch.start_notify()?;
        }
    }

    Ok(())
}

fn connect_joystick(
    bt_session: &BluetoothSession,
    device: &BluetoothDevice,
) -> Result<(), Box<dyn Error>> {
    if !device.is_paired()? {
        let r = device.pair();
        println!("Pair returns {:?}", r);
    } else {
        println!("Device paired");
    }

    if let Err(e) = device.connect(10000) {
        println!("Failed to connect {}: {:?}", device.get_id(), e);
    } else {
        let r = enable_joystick_notify(&bt_session, &device);
        println!(
            "Connect success! {}. Enable notify: {:?}",
            device.get_id(),
            r
        );
    }

    Ok(())
}

fn main() {
    let bt_session = &BluetoothSession::create_session(None).unwrap();

    let joysticks = get_joysticks_with_event(bt_session, 10).unwrap();
    let joysticks_paired = get_joysticks_paired(bt_session).unwrap();

    if joysticks.len() == 0 && joysticks_paired.len() == 0 {
        eprintln!("No joysticks found, exit");
        return;
    }

    for device in joysticks.iter() {
        let r = connect_joystick(bt_session, &device);
        println!("{:?} result {:?}", device, r);
    }

    for device in joysticks_paired.iter() {
        let r = connect_joystick(bt_session, &device);
        println!("{:?} result {:?}", device, r);
    }

    loop {
        for event in bt_session.incoming(1000).map(BluetoothEvent::from) {
            println!("recv: {:?}", event);
        }
    }
}
