extern crate blurz;

static BATTERY_SERVICE_UUID: &'static str = "0000180f-0000-1000-8000-00805f9b34fb";
static BELL_CONTROLLER_SERVICE_UUID: &'static str = "0000885a-0000-1000-8000-00805f9b34fb";

use blurz::bluetooth_adapter::BluetoothAdapter as Adapter;
use blurz::bluetooth_device::BluetoothDevice as Device;
use blurz::bluetooth_discovery_session::BluetoothDiscoverySession as DiscoverySession;
use blurz::bluetooth_gatt_characteristic::BluetoothGATTCharacteristic as Characteristic;
use blurz::bluetooth_gatt_descriptor::BluetoothGATTDescriptor as Descriptor;
use blurz::bluetooth_gatt_service::BluetoothGATTService as Service;
use blurz::bluetooth_session::BluetoothSession as Session;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::os::unix::io::FromRawFd;
use std::thread;
use std::time::Duration;

fn test2() -> Result<(), Box<dyn Error>> {
    let bt_session = &Session::create_session(None)?;
    let adapter: Adapter = Adapter::init(bt_session)?;
    let session = DiscoverySession::create_session(&bt_session, adapter.get_id())?;
    session.start_discovery()?;
    //let mut devices = vec!();
    for _ in 0..5 {
        let devices = adapter.get_device_list()?;

        thread::sleep(Duration::from_millis(2000));
    }
    session.stop_discovery()?;
    let devices = adapter.get_device_list()?;
    if devices.is_empty() {
        return Err(Box::from("No device found"));
    }
    println!("{} device(s) found", devices.len());
    let mut device: Device = Device::new(bt_session, "".to_string());
    for d in devices {
        device = Device::new(bt_session, d.clone());
        if let Ok(name) = device.get_alias() {
            // println!("Device: {} {:?}", device.get_id(), device.get_alias());
            let uuids = device.get_uuids()?;

            if name.contains("bell") {
                println!("found bell device: {:?}", device);
                println!("Device uuids: {:?}", uuids);
                let r = device.connect(10000);
                if r.is_err() {
                    eprintln!("conn err: {:?}", r);
                }

                if device.is_connected()? {
                    println!("Device connected: {:?}", device);
                    println!("checking gatt...");
                    // We need to wait a bit after calling connect to safely
                    // get the gatt services
                    thread::sleep(Duration::from_millis(2000));
                    match device.get_gatt_services() {
                        Ok(services) => println!("GATT services: {:?}", services),
                        Err(e) => eprintln!("{:?}", e),
                    }
                    break;
                } else {
                    println!("could not connect");
                }
            }
        }
        println!("");
    }
    adapter.stop_discovery().ok();
    if !device.is_connected()? {
        return Err(Box::from("No connectable device found"));
    }

    let services = device.get_gatt_services()?;

    let mut ch = None;
    for service in services {
        let s = Service::new(bt_session, service.clone());
        println!("S uuid: {:?}", s.get_uuid());
        let characteristics = s.get_gatt_characteristics()?;
        for characteristic in characteristics {
            let c = Characteristic::new(bt_session, characteristic.clone());
            println!("C uuid: {:?}", c.get_uuid());
            println!("Value: {:?}", c.read_value(None));
            if let Ok(uuid) = c.get_uuid() {
                if uuid == BELL_CONTROLLER_SERVICE_UUID {
                    ch = Some(c.clone());
                    break;
                }
            }

            let descriptors = c.get_gatt_descriptors()?;
            for descriptor in descriptors {
                let d = Descriptor::new(bt_session, descriptor.clone());
                println!("D {:?}", d);
                println!("Value: {:?}", d.read_value(None));
            }
        }
    }

    if let Some(ch) = ch {
        if let Ok((fd, mtu)) = ch.acquire_notify() {
            println!("acquire_notify success");
            // ? how to read notifications from fd?
            let f = unsafe { File::from_raw_fd(fd.into_fd()) };
            // let mut buf = vec![0; mtu as usize];
            // f.read(&mut contents)?;
        }

    // let r = ch.start_notify();
    // println!("start_notify {:?}", r);

    // loop {
    //     if let Ok(r) = ch.read_value(None) {
    //         println!("get: {:?}", r);
    //     }

    //     thread::sleep(Duration::from_secs(100));
    // }
    } else {
        eprintln!("Character not found!");
    }

    let r = device.disconnect();
    println!("disconn {:?}", r);
    Ok(())
}

fn main() {
    match test2() {
        Ok(_) => (),
        Err(e) => println!("Error: {:?}", e),
    }
}
