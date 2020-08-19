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

const MMC_SERVICE_UUID: &str = "1809";
const MMC_CHAR_UUID: &str = "2a1e";
const MMC_TITLE: &str = "MMC";

const UUID_REGEX: &str = r"([0-9a-f]{4})([0-9a-f]{4})-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}";

lazy_static! {
    static ref RE: Regex = Regex::new(UUID_REGEX).unwrap();
}

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
            _ => format!("{:x?}", value),
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

fn parse_mmc_data(data: Box<[u8]>) -> Option<(f32, f32, f32, f32)> {
    if data.len() == 6 {
        let mut t0: f32 = data[2] as f32 * 256.0 + data[1] as f32;
        let mut offset: f32 = 0.0;
        let mut t1: f32 = 0.0;
        let mut t4: f32 = t0;

        if data[3] != 0xf2 || data[4] != 0x7f {
            t1 = data[4] as f32 * 256.0 + data[3] as f32;
            let diff = t0 - t1;
            offset = diff / 2.0;

            if diff > 0.0 {
                let mut off = diff;
                while off > 200.0 {
                    off = off - 100.0;
                }

                if off < 100.0 {
                    off += 50.0;
                }

                t4 = t0 + off;
            }

            if offset > 1.0 {
                offset = 1.0;
            }
        }

        let mut toff = t0 + offset;
        t1 = t1 / 100.0;
        t0 = t0 / 100.0;
        toff = toff / 100.0;
        t4 = t4 / 100.0;

        println!("t0: {}, t1: {}, toff: {}, t4: {}", t0, t1, toff, t4);
        return Some((t0, t1, toff, t4));
    }
    None
}

fn main() {
    let bt_session = &BluetoothSession::create_session(None).unwrap();
    let adapter: BluetoothAdapter = BluetoothAdapter::init(bt_session).unwrap();
    let adapter_id = adapter.get_id();
    // 创建蓝牙搜索的Session
    let discover_session =
        BluetoothDiscoverySession::create_session(&bt_session, adapter_id).unwrap();
    // 开始扫描设备
    discover_session.start_discovery().unwrap();
    // 等待几秒
    thread::sleep(Duration::from_secs(5));
    // 获取设备列便
    let device_list = adapter.get_device_list().unwrap();
    // 结束扫描
    discover_session.stop_discovery().unwrap();

    for device_path in device_list {
        let device = BluetoothDevice::new(bt_session, device_path.to_string());
        println!(
            "Device: {:?} Name: {:?}, RSSI: {:?}",
            device_path,
            device.get_name().ok(),
            device.get_rssi().ok()
        );
    }

    let device = BluetoothDevice::new(
        bt_session,
        String::from("/org/bluez/hci0/dev_00_81_F9_DF_B0_40"), // mmc
    );

    if let Err(e) = device.connect(10000) {
        println!("Failed to connect {:?}: {:?}", device.get_id(), e);
    } else {
        println!("Connected!");
        // We need to wait a bit after calling connect to safely
        // get the gatt services
        thread::sleep(Duration::from_secs(5));

        // print services, characteristics and descriptors
        // explore_device(&device, bt_session);

        let service = get_service(MMC_SERVICE_UUID, &device, bt_session).unwrap();
        let ch = get_characteritic(MMC_CHAR_UUID, &service, bt_session).unwrap();
        ch.start_notify().unwrap();
        loop {
            for event in bt_session.incoming(1000).map(BluetoothEvent::from) {
                if let Some(event) = event {
                    println!("recv: {:?}", event);
                    match event {
                        Value { object_path, value } => {
                            if let Some((raw, _, _, t)) = parse_mmc_data(value) {
                                println!("Raw t: {}, calibrated: {}", raw, t);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
