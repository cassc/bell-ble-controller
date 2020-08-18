/// Run this command to turn on bluetooth first:
/// bluetoothctl power on

static BATTERY_SERVICE_UUID: &'static str = "0000180f-0000-1000-8000-00805f9b34fb";
static BELL_CONTROLLER_SERVICE_UUID: &'static str = "0000885a-0000-1000-8000-00805f9b34fb";

use blurz::bluetooth_adapter::BluetoothAdapter;
use blurz::bluetooth_device::BluetoothDevice;
use blurz::bluetooth_discovery_session::BluetoothDiscoverySession;
use blurz::bluetooth_event::BluetoothEvent;
use blurz::bluetooth_gatt_characteristic::BluetoothGATTCharacteristic;
use blurz::bluetooth_gatt_descriptor::BluetoothGATTDescriptor;
use blurz::bluetooth_gatt_service::BluetoothGATTService;
use blurz::bluetooth_session::BluetoothSession;
use lazy_static::lazy_static;
use regex::Regex;
use std::str;
use std::thread;
use std::time::Duration;

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

fn read_notification(characteristic: &BluetoothGATTCharacteristic, session: &BluetoothSession) {
    // let temp_humidity = BluetoothGATTCharacteristic::new(
    //     session,
    //     String::from("/org/bluez/hci0/dev_A4_C1_38_15_03_55/service0021/char0035"),
    // );
    characteristic.start_notify().unwrap();
    loop {
        for event in BluetoothSession::create_session(None)
            .unwrap()
            .incoming(1000)
            .map(BluetoothEvent::from)
        {
            println!("event {:?}", event);
        }
    }
}

fn main() {
    let bt_session = &BluetoothSession::create_session(None).unwrap();
    let adapter: BluetoothAdapter = BluetoothAdapter::init(bt_session).unwrap();
    let adapter_id = adapter.get_id();
    let discover_session =
        BluetoothDiscoverySession::create_session(&bt_session, adapter_id).unwrap();
    discover_session.start_discovery().unwrap();
    thread::sleep(Duration::from_secs(5));
    let device_list = adapter.get_device_list().unwrap();
    discover_session.stop_discovery().unwrap();

    for device_path in device_list {
        let device = BluetoothDevice::new(bt_session, device_path.to_string());
        println!(
            "Device: {:?} Name: {:?}",
            device_path,
            device.get_name().ok()
        );
    }

    let device = BluetoothDevice::new(
        bt_session,
        String::from("/org/bluez/hci0/dev_00_81_F9_DF_B0_40"), // mmc
    );

    // if !device.is_paired().unwrap() {
    //     let r = device.pair();
    //     println!("Pair returns {:?}", r);
    // } else {
    //     println!("Device paired");
    // }

    if let Err(e) = device.connect(10000) {
        println!("Failed to connect {:?}: {:?}", device.get_id(), e);
    } else {
        println!("Connected!");
        // We need to wait a bit after calling connect to safely
        // get the gatt services
        thread::sleep(Duration::from_secs(5));

        // print services, characteristics and descriptors
        explore_device(&device, bt_session);

        println!(
            "--------------------------------------------------------------------------------"
        );

        // print to notifications
        // let ch = BluetoothGATTCharacteristic::new(
        //     bt_session,
        //     String::from("/org/bluez/hci0/dev_A4_C1_38_15_03_55/service0021/char0035"),
        // );

        // let uuid_service = "8850";
        // let uuid_characteritic = "885a";

        let uuid_service = "1809";
        let uuid_characteritic = "2a1e";

        if let Some(service) = get_service(uuid_service, &device, bt_session) {
            let session = bt_session;
            let w_ch = vec![
                "/org/bluez/hci0/dev_10_38_C1_30_7B_03/service0020/char0025",
                "/org/bluez/hci0/dev_10_38_C1_30_7B_03/service0020/char0023",
                "/org/bluez/hci0/dev_10_38_C1_30_7B_03/service0020/char0021",
            ];
            // for s in w_ch {
            //     let ch = BluetoothGATTCharacteristic::new(&session, String::from(s));
            //     ch.write_value(vec![0x01, 0x00], None).unwrap();
            // }

            if let Some(ch) = get_characteritic(uuid_characteritic, &service, session) {
                ch.start_notify().unwrap();

                loop {
                    // let session = BluetoothSession::create_session(None).unwrap();
                    for event in session.incoming(1000).map(BluetoothEvent::from) {
                        println!("recv: {:?}", event);
                    }
                }
                // use std::io::Read;
                // let fd = ch.acquire_notify().unwrap();
            }

        // let ch_a = BluetoothGATTCharacteristic::new(
        //     &session,
        //     String::from("/org/bluez/hci0/dev_10_38_C1_30_7B_03/service0020/char002a"),
        // );
        // let ch_b = BluetoothGATTCharacteristic::new(
        //     &session,
        //     String::from("/org/bluez/hci0/dev_10_38_C1_30_7B_03/service0020/char0027"),
        // );

        // ch_a.start_notify().unwrap();
        // ch_b.start_notify().unwrap();

        // loop {
        //     let session = BluetoothSession::create_session(None).unwrap();
        //     for event in session.incoming(1000).map(BluetoothEvent::from) {
        //         println!("recv: {:?}", event);
        //     }
        // }

        // character uuid 00002a50, battery?
        // {
        //     let ch = BluetoothGATTCharacteristic::new(
        //         &session,
        //         String::from("/org/bluez/hci0/dev_10_38_C1_30_7B_03/service000c/char000d"),
        //     );

        //     loop {
        //         let r = ch.read_value(None).unwrap();
        //         println!("Read: {:?}", r);
        //         thread::sleep(Duration::from_millis(200));
        //     }
        // }
        } else {
            eprintln!("Service not found!");
        }

        // Interact with device
        device.disconnect().unwrap();
    }
}
