Great — both optimizations are **very useful on the LicheeRV Nano**, because the SG2002 boards usually ship with **64–128 MB RAM** and relatively slow SD I/O. With some tuning you can reach **~2–3 s boot to UI** and keep runtime memory around **50–60 MB**.

I'll break it into two parts:

1. **Fast boot (≈2–3 s)**
2. **Low memory footprint (~55 MB)**

---

# 1. Fast Boot Optimization

The boot chain is:

```
BootROM
 → FSBL
 → OpenSBI
 → U-Boot
 → Linux
 → init
 → UI
```

You cannot change the **FSBL**, but you can optimize everything after it.

---

## 1.1 Optimize U-Boot

The project **U-Boot** can easily waste ~2 seconds waiting for boot menus.

### Disable boot delay

In your U-Boot config:

```c
CONFIG_BOOTDELAY=0
```

Or from the environment:

```bash
setenv bootdelay 0
saveenv
```

---

### Disable unused subsystems

In U-Boot defconfig disable:

```
CONFIG_CMD_NET=n
CONFIG_CMD_DHCP=n
CONFIG_CMD_PXE=n
CONFIG_CMD_NFS=n
```

This removes unnecessary network initialization.

---

### Boot directly from fixed device

Example boot command:

```
bootcmd=load mmc 0:1 ${kernel_addr_r} Image; booti ${kernel_addr_r} - ${fdt_addr_r}
```

Avoid filesystem scanning.

---

### Expected savings

| Stage           | Time saved |
| --------------- | ---------- |
| U-Boot menu     | ~1 s       |
| network probing | ~0.5 s     |

---

# 2. Kernel Boot Optimization

Using **Linux kernel 6.6**.

---

## 2.1 Disable slow kernel features

In kernel config:

```
CONFIG_PRINTK_TIME=n
CONFIG_DEBUG_KERNEL=n
CONFIG_KALLSYMS=n
CONFIG_SLUB_DEBUG=n
CONFIG_FTRACE=n
CONFIG_PROFILING=n
```

These reduce boot overhead.

---

## 2.2 Reduce driver probing

Disable unused subsystems:

```
CONFIG_SCSI=n
CONFIG_PCI=n
CONFIG_FIREWIRE=n
CONFIG_MEDIA_SUPPORT=n
```

You probably only need:

```
USB
MMC
INPUT
DRM
SOUND
NET
```

---

## 2.3 Quiet boot

Add to kernel cmdline:

```
quiet loglevel=3
```

This reduces console output delays.

---

## 2.4 Use init instead of systemd

Buildroot defaults to **BusyBox init**, which is perfect.

The project **BusyBox** is extremely lightweight.

Boot scripts run much faster.

---

## Expected Boot Time

Typical optimized timing:

| Stage     | Time    |
| --------- | ------- |
| FSBL      | ~0.4 s  |
| OpenSBI   | ~0.05 s |
| U-Boot    | ~0.5 s  |
| Kernel    | ~1.2 s  |
| Userspace | ~0.5 s  |

Total:

```
≈2.5 seconds to UI
```

---

# 3. Memory Optimization (~55 MB runtime)

Your biggest memory consumers will be:

1. kernel
2. graphics stack
3. Rust UI

---

## 3.1 Kernel memory reduction

In kernel config:

```
CONFIG_DEBUG_INFO=n
CONFIG_KALLSYMS=n
CONFIG_MODULES=n
CONFIG_BPF=n
CONFIG_KPROBES=n
```

Also shrink networking:

```
CONFIG_IPV6=n
CONFIG_NETFILTER=n
```

Savings:

```
~6–10 MB RAM
```

---

## 3.2 Filesystem optimization

Prefer:

```
squashfs
```

instead of ext4.

Buildroot option:

```
BR2_TARGET_ROOTFS_SQUASHFS=y
```

Advantages:

* smaller rootfs
* lower RAM cache pressure

---

## 3.3 Framebuffer instead of Wayland

Your UI uses **Slint UI Toolkit**, which supports a **software framebuffer renderer**.

This avoids heavy stacks like:

* Wayland
* Weston
* X11

Memory comparison:

| Graphics stack       | RAM      |
| -------------------- | -------- |
| Framebuffer + Slint  | ~8 MB    |
| Wayland + compositor | 30–60 MB |

Huge difference.

---

## 3.4 Rust binary optimization

For **Rust** builds:

In `Cargo.toml`:

```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
opt-level = "z"
strip = true
```

Binary size reduction:

```
~40–60%
```

---

## 3.5 Disable glibc locales

In Buildroot:

```
BR2_ENABLE_LOCALE_PURGE=y
```

Removes ~10 MB of files.

---

# 4. Minimal Runtime Architecture

Optimized system:

```
Linux 6.6
   │
fbdev / DRM
   │
libinput
   │
Slint UI
```

No:

* Wayland
* X11
* Qt
* systemd

---

# 5. Example Runtime Memory

Typical numbers:

| Component        | RAM   |
| ---------------- | ----- |
| Kernel           | 18 MB |
| Filesystem cache | 10 MB |
| Slint UI         | 12 MB |
| libinput         | 2 MB  |
| audio            | 3 MB  |

Total:

```
≈45–55 MB
```

Works well even on **64 MB boards**.

---

# 6. Boot-to-UI Trick

Start the UI directly from BusyBox init.

`/etc/inittab`:

```
::sysinit:/etc/init.d/rcS
::respawn:/usr/bin/my_ui
```

Your Rust UI launches immediately after boot.

---

# Final Result

Your stack will look like this:

```
FSBL
OpenSBI
U-Boot
Linux 6.6
BusyBox init
Rust + Slint UI
```

with:

```
~2–3 s boot
~50 MB RAM usage
```

---

✅ If you'd like, I can also show you something **very helpful specifically for your board**:

1. **How to measure the exact boot time of every stage (FSBL → UI)** using kernel timestamps.
2. **How to shrink the whole SD card image to under 80 MB** while keeping Wi-Fi, Bluetooth, audio, and the UI.
