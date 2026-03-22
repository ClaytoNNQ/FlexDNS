use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedObjectPath;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct DhbcDeviceInfo {
    pub interface: String,
    pub dns_servers: Vec<Ipv4Addr>,
}

pub fn get_dhbc_info() -> Option<Vec<DhbcDeviceInfo>> {
    let conn = Connection::system().ok()?;
    let nm_proxy = Proxy::new(&conn, "org.freedesktop.NetworkManager", "/org/freedesktop/NetworkManager", "org.freedesktop.NetworkManager").ok()?;

    let device_paths: Vec<OwnedObjectPath> = nm_proxy.call("GetDevices", &()).ok()?;
    let mut results = Vec::new();

    for path in device_paths {
        let dev_proxy = match Proxy::new(&conn, "org.freedesktop.NetworkManager", path.as_str(), "org.freedesktop.NetworkManager.Device") {
            Ok(p) => p,
            Err(_) => continue,
        };

        let state: u32 = match dev_proxy.get_property("State") {
            Ok(v) => v,
            Err(_) => continue,
        };

        if state != 100 {
            continue;
        }

        let iface: String = match dev_proxy.get_property("Interface") {
            Ok(v) => v,
            Err(_) => continue,
        };

        let ip4_path: OwnedObjectPath = match dev_proxy.get_property("Ip4Config") {
            Ok(v) => v,
            Err(_) => {
                results.push(DhbcDeviceInfo { interface: iface, dns_servers: Vec::new()});
                continue;
            }
        };

        if ip4_path.as_str() == "/" {
            results.push(DhbcDeviceInfo { interface: iface, dns_servers: Vec::new()});
            continue;
        }

        let ip4_proxy = match Proxy::new(&conn,"org.freedesktop.NetworkManager", ip4_path.as_str(), "org.freedesktop.NetworkManager.IP4Config") {
            Ok(p) => p,
            Err(_) => {
                results.push(DhbcDeviceInfo { interface: iface, dns_servers: Vec::new()});
                continue;
            }
        };

        let dns_vec: Vec<u32> = ip4_proxy.get_property("Nameservers").unwrap_or_default();
        if dns_vec.is_empty() { continue; }
        let dns_servers: Vec<Ipv4Addr> =dns_vec.into_iter().map(|d| Ipv4Addr::from(u32::from_be(d))).collect();

        results.push(DhbcDeviceInfo {
            interface: iface,
            dns_servers,
        });
    }

    if results.is_empty() {
        return None;
    }

    results.sort_by_key(|d| {
        if d.interface.starts_with("tun") || d.interface.starts_with("tap") || d.interface.starts_with("wg") {
            0
        } else if d.interface.starts_with("eth") || d.interface.starts_with("enp") || d.interface.starts_with("ens") {
            1
        } else if d.interface.starts_with("wlan") {
            2
        } else if d.interface.starts_with("wwan") {
            3
        } else if d.interface.starts_with("lo") {
            5
        } else {
            4
        }
    });

    Some(results)
}