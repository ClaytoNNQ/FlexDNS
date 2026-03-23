# FlexDns
>
>
>
**FlexDns** is a minimal, high-performance DNS resolver for Linux systems. Designed to be lighter and faster than `systemd-resolved`, it uses significantly less CPU and memory while providing a fully functional DNS service. FlexDns also supports **encrypted DNS** via **DoH (DNS over HTTPS)** and **DoT (DNS over TLS)**, ensuring privacy and security without sacrificing performance.


---

>
> [!NOTE]  
> **Features:**  
> - Minimal and optimized for low resource consumption.  
> - Supports **DoH** and **DoT** protocols for secure queries.  
> - Provides fallback DNS when system resolver is unset or misconfigured.  
> - Supports asynchronous DNS registration with editor support.

---

## Installation

1. Download the Linux release archive => [flex-dns-x86_64.tar.gz](https://raw.githubusercontent.com/ClaytoNNQ/FlexDNS/main/flex-dns-x86_64.tar.gz)

### Extract the archive

---

### Extract the archive

```bash
tar -xvf flexdns-linux-x86_64.tar
cd flexdns-linux-x86_64
```

### Run the installation script with root privileges

```bash
sudo bash ./install.sh
```

>
> [!CAUTION]
> The installation script blocks DHCP-provided DNS entries from NetworkManager and `systemd-resolved`.
> It automatically stops and disables `systemd-resolved` if running.
> If the system resolver is not set, FlexDns uses the last available fallback DNS.
> Misconfigured system resolver entries are automatically corrected by FlexDns.

>
> [!NOTE]
> For registration or editing of DNS entries, **avoid editors that perform asynchronous writes**. Use a standard synchronous editor to ensure changes are applied correctly.
> FlexDns automatically detects network changes; **no service restart is required**.

---

## Configuration

* Default DNS configuration is stored in:

```text
/etc/flexdns/resolv.conf
```

* FlexDns updates the system resolver automatically on network changes.
* If the resolver is unset, the fallback DNS is applied automatically.
* Misconfigured resolver entries are restored to defaults without user intervention.

---

## Usage

After installation, FlexDns runs transparently in the background, efficiently handling DNS queries without further user action.

---

## License

This project is licensed under the [MIT License](./LICENSE).
