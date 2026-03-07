Adding **Wi-Fi and Bluetooth** is very doable on the LicheeRV Nano with a mainline kernel, but the **specific module on the board determines how easy it is**. The Nano variants typically ship with a Realtek combo chip.

Most boards use one of these:

| Chip      | Interface      | Linux driver |
| --------- | -------------- | ------------ |
| RTL8723DS | SDIO + UART BT | `rtl8723ds`  |
| RTL8723BS | SDIO + UART BT | `rtl8723bs`  |
| RTL8821CS | SDIO + UART BT | `rtl8821cs`  |

These chips are made by **Realtek**.

The important detail: **Wi-Fi and Bluetooth are separate devices inside the same chip**.

```
SDIO → WiFi
UART → Bluetooth
```

---

# 1. Wi-Fi Support

The Realtek SDIO drivers are partially upstream.

Kernel 6.6 includes:

```
CONFIG_RTL8XXXU
CONFIG_RTW88
CONFIG_RTL8723BS
```

However, many boards still use **vendor Realtek drivers** because:

* SDIO firmware loading quirks
* power sequencing differences

In Buildroot you typically enable:

```
BR2_PACKAGE_WPA_SUPPLICANT=y
BR2_PACKAGE_IW=y
BR2_PACKAGE_LINUX_FIRMWARE=y
```

Then load firmware from:

```
/lib/firmware/rtlwifi/
```

Example connection:

```
wpa_passphrase SSID PASSWORD > /etc/wpa_supplicant.conf
wpa_supplicant -B -i wlan0 -c /etc/wpa_supplicant.conf
udhcpc -i wlan0
```

---

# 2. Bluetooth Support

Bluetooth on these modules is connected via **UART HCI**.

Linux uses the **HCI UART driver**.

Kernel config:

```
CONFIG_BT=y
CONFIG_BT_HCIUART=y
CONFIG_BT_HCIUART_RTL=y
```

Userland tools in Buildroot:

```
BR2_PACKAGE_BLUEZ5_UTILS=y
```

Firmware is loaded using:

```
/lib/firmware/rtl_bt/
```

Start sequence:

```
hciattach /dev/ttyS1 rtk_h5
hciconfig hci0 up
```

Then verify:

```
hciconfig
```

---

# 3. Device Tree Requirements

This is the only place where you might need to modify things when moving to kernel 6.6.

Typical nodes:

```
wifi@1 {
    compatible = "realtek,rtl8723ds";
    reg = <1>;
};
```

Bluetooth:

```
bluetooth {
    compatible = "realtek,rtl8723bs-bt";
    uart = <&uart1>;
};
```

If the vendor DTS already works, you can often **reuse it with minimal changes**.

---

# 4. Buildroot Networking Stack

Minimal packages:

```
BR2_PACKAGE_WPA_SUPPLICANT
BR2_PACKAGE_BLUEZ5_UTILS
BR2_PACKAGE_IW
BR2_PACKAGE_IPROUTE2
```

Optional but useful:

```
BR2_PACKAGE_CONNMAN
```

The project **ConnMan** is lightweight and works well on embedded systems.

---

# 5. Performance Expectations

On these small Realtek SDIO chips:

| Feature          | Typical      |
| ---------------- | ------------ |
| Wi-Fi throughput | 20–40 Mbps   |
| Bluetooth        | BLE + audio  |
| Latency          | fine for IoT |

Good enough for:

* remote UI control
* OTA updates
* telemetry
* mobile apps

---

# 6. Potential Gotchas

### Power sequencing

Some boards require a GPIO reset line.

### Firmware blobs

You must ship:

```
rtl_bt/*.bin
rtlwifi/*.bin
```

### Driver maturity

Some Realtek SDIO drivers are not perfect upstream.

---

# 7. Recommended Final Stack

Your full software stack could look like this:

```
FSBL (vendor)
OpenSBI
U-Boot (mainline)
Linux 6.6

Buildroot
 ├─ ALSA
 ├─ USB audio
 ├─ libinput
 ├─ Rust
 ├─ Slint UI
 ├─ WiFi (wpa_supplicant)
 └─ Bluetooth (BlueZ)
```

Hardware supported:

* USB-C audio
* touch display
* Wi-Fi
* Bluetooth
* Rust UI

All without the vendor multimedia stack.

---

✅ **Bottom line:**
Your requirements **fit very well with a mainline 6.6 kernel** and a clean Buildroot environment.

---

If you'd like, I can also show you:

1️⃣ a **minimal Buildroot defconfig for the LicheeRV Nano with Wi-Fi, Bluetooth, Rust and Slint**
2️⃣ how to **extract the vendor DTS and convert it to a mainline-compatible one**
3️⃣ how to **package FSBL + OpenSBI + U-Boot into `fip.bin` automatically in Buildroot** (this is the trickiest part of the whole system).
