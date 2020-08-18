/// Run this command to turn on bluetooth first:
/// bluetoothctl power on

static BELL_CONTROLLER_SERVICE_UUID: &'static str = "00008850-0000-1000-8000-00805f9b34fb";
static BELL_CONTROLLER_CHARACTER_UUID: &'static str = "0000885a-0000-1000-8000-00805f9b34fb";
use blurz::bluetooth_adapter::BluetoothAdapter;
use blurz::bluetooth_device::BluetoothDevice;
use blurz::bluetooth_discovery_session::BluetoothDiscoverySession;
use blurz::bluetooth_event::BluetoothEvent;
use blurz::bluetooth_event::BluetoothEvent::{Connected, ServicesResolved, Value, RSSI};
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

#[derive(Clone, Debug)]
pub struct JoystickKeyEvent {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    i: bool,
    ii: bool,
    a: bool,
    b: bool,
    c: bool,
    d: bool,
    l1: bool,
    l2: (u8, bool),
    r1: bool,
    r2: (u8, bool),
    rl: (u8, u8),
    rr: (u8, u8),
}

#[derive(Clone, Debug)]
pub enum JoystickEvent {
    Key(String, JoystickKeyEvent),
    Home(String, bool),
}

// 手柄10字节对应的按键
// 方向键不可组合
const JS_UP: (usize, u8) = (8, 1);
const JS_DOWN: (usize, u8) = (8, 5);
const JS_LEFT: (usize, u8) = (8, 7);
const JS_RIGHT: (usize, u8) = (8, 3);
// const JS_UP_LEFT: (usize, u8) = (8, 8);
// const JS_UP_RIGHT: (usize, u8) = (8, 2);
// const JS_DOWN_LEFT: (usize, u8) = (8, 6);
// const JS_DOWN_RIGHT: (usize, u8) = (8, 4);

// 其他键可组合
const JS_I: (usize, u8) = (7, 4);
const JS_II: (usize, u8) = (7, 8);
const JS_A: (usize, u8) = (6, 1);
const JS_B: (usize, u8) = (6, 2);
const JS_C: (usize, u8) = (6, 8);
const JS_D: (usize, u8) = (6, 16);
const JS_L1: (usize, u8) = (6, 0x40);
const JS_R1: (usize, u8) = (6, 0x80);

// 模拟量按键，后一个数字表示是否完全按下
const JS_L2: (usize, usize, u8) = (4, 7, 1);
const JS_R2: (usize, usize, u8) = (5, 7, 2);

// 左右侧旋钮
const JS_RL: (usize, usize) = (0, 1);
const JS_RR: (usize, usize) = (2, 3);

const BELL_HOME_DOWN: [u8; 3] = [8, 0, 0];

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

fn handle_ble_event(event: Option<BluetoothEvent>) -> Option<JoystickEvent> {
    if let Some(event) = event {
        match event {
            Value { object_path, value } => {
                println!("{:x?}", value);
                let len = value.len();
                if len == 10 {
                    let up = value[JS_UP.0] == JS_UP.1;
                    let down = value[JS_DOWN.0] == JS_DOWN.1;
                    let left = value[JS_LEFT.0] == JS_LEFT.1;
                    let right = value[JS_RIGHT.0] == JS_RIGHT.1;
                    let i = value[JS_I.0] & JS_I.1 > 0;
                    let ii = value[JS_II.0] & JS_II.1 > 0;
                    let a = value[JS_A.0] & JS_A.1 > 0;
                    let b = value[JS_B.0] & JS_B.1 > 0;
                    let c = value[JS_C.0] & JS_C.1 > 0;
                    let d = value[JS_D.0] & JS_D.1 > 0;
                    let l1 = value[JS_L1.0] & JS_L1.1 > 0;
                    let r1 = value[JS_R1.0] & JS_R1.1 > 0;
                    let l2 = (value[JS_L2.0], value[JS_L2.1] & JS_L2.2 > 0);
                    let r2 = (value[JS_R2.0], value[JS_R2.1] & JS_R2.2 > 0);
                    let rl = (value[JS_RL.0], value[JS_RL.1]);
                    let rr = (value[JS_RR.0], value[JS_RR.1]);

                    return Some(JoystickEvent::Key(
                        object_path,
                        JoystickKeyEvent {
                            up,
                            down,
                            left,
                            right,
                            i,
                            ii,
                            a,
                            b,
                            c,
                            d,
                            l1,
                            l2,
                            r1,
                            r2,
                            rl,
                            rr,
                        },
                    ));
                } else if len == 3 {
                    let down = *value == BELL_HOME_DOWN;
                    return Some(JoystickEvent::Home(object_path, down));
                }
            }
            Connected {
                object_path,
                connected,
            } => {
                println!(
                    "Device {}connected {}",
                    object_path,
                    if connected { "" } else { "dis" }
                );
            }

            ServicesResolved {
                object_path,
                services_resolved,
            } => {}

            _ => {}
        }
    }
    None
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
            if let Some(event) = handle_ble_event(event) {
                println!("recv key event: {:?}", event);
            }
        }
    }
}
