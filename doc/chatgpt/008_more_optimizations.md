Good — these two techniques make embedded development **much easier to iterate and optimize**, especially when working with custom Buildroot systems.

We'll cover:

1. **Precise boot-time measurement (FSBL → UI)**
2. **Shrinking the final SD card image (<80 MB)**

Both integrate cleanly with **Buildroot**, **U-Boot**, and the **Linux kernel**.

---

# 1. Measuring Boot Time Precisely

If you don't measure boot stages, it's hard to optimize them.

The goal is to get timing like:

```text
FSBL        0.40s
OpenSBI     0.05s
U-Boot      0.55s
Kernel      1.20s
Userspace   0.60s
UI start    2.80s
```

---

# 1.1 Enable Kernel Timestamping

Enable in kernel config:

```text
CONFIG_PRINTK_TIME=y
CONFIG_BOOT_PRINTK_DELAY=n
```

Then boot log will show:

```text
[    0.000000] Linux version 6.6.x
[    0.142381] clocksource: timer
[    0.901212] Freeing init memory
```

This tells you how long the kernel took.

---

# 1.2 Measure Userspace Startup

Add a timestamp print to your init script.

Example `/etc/init.d/S01bootlog`:

```sh
#!/bin/sh
echo "userspace start $(cut -d' ' -f1 /proc/uptime)"
```

Output example:

```text
userspace start 1.33
```

Now you know:

```
kernel finished at 1.33 s
```

---

# 1.3 Measure UI Start

In your Rust program (using **Rust**):

```rust
use std::time::Instant;

fn main() {
    println!("UI starting at {:?}", Instant::now());
}
```

Even simpler:

```rust
println!("ui-start {}", std::fs::read_to_string("/proc/uptime").unwrap());
```

Now your boot log shows the **full pipeline timing**.

---

# 1.4 Measure U-Boot Time

Enable timestamp prints in **U-Boot**:

```text
CONFIG_BOOTSTAGE=y
CONFIG_BOOTSTAGE_REPORT=y
```

Then run:

```
bootstage report
```

Example:

```
U-Boot start     0 ms
MMC init       120 ms
Kernel load    310 ms
bootm start    420 ms
```

This tells you where time is spent.

---

# 1.5 Best Tool for Boot Profiling

The kernel tool **bootchart** is extremely useful.

Enable in Buildroot:

```
BR2_PACKAGE_BOOTCHART=y
```

It produces a graph like:

```
kernel → init → services → UI
```

This helps identify slow services.

---

# 2. Shrinking the System Image (<80 MB)

Embedded images often waste space due to defaults.

Let's reduce them.

---

# 2.1 Use SquashFS Root Filesystem

Instead of ext4.

In Buildroot:

```
BR2_TARGET_ROOTFS_SQUASHFS=y
```

Advantages:

| Filesystem | Typical size |
| ---------- | ------------ |
| ext4       | ~120 MB      |
| squashfs   | ~35 MB       |

Huge improvement.

---

# 2.2 Strip Binaries

Buildroot option:

```
BR2_STRIP_strip=y
```

Removes debug symbols.

Example:

| Binary  | Before | After |
| ------- | ------ | ----- |
| Rust UI | 18 MB  | 6 MB  |

---

# 2.3 Remove Unused BusyBox Tools

BusyBox contains many commands.

Open config:

```
make busybox-menuconfig
```

Disable things like:

```
awk
vi
diff
ftp
telnet
```

This saves **several MB**.

---

# 2.4 Remove Kernel Modules

Instead of modules:

```
CONFIG_MODULES=n
```

Compile drivers directly into kernel.

Benefits:

* smaller rootfs
* faster boot
* fewer files

---

# 2.5 Reduce glibc Footprint

In Buildroot:

```
BR2_ENABLE_LOCALE_PURGE=y
BR2_GENERATE_LOCALE=""
```

Locales consume a lot of space.

Savings:

```
~10–15 MB
```

---

# 2.6 Compress Kernel

Kernel compression option:

```
CONFIG_KERNEL_LZ4=y
```

Comparison:

| Compression | Boot speed |
| ----------- | ---------- |
| gzip        | slower     |
| LZ4         | fastest    |

Better for embedded systems.

---

# 2.7 Smaller SD Card Image

Update your `genimage.cfg`.

Example:

```cfg
image sdcard.img {
  hdimage {}

  partition boot {
    image = "boot.vfat"
    size = 16M
  }

  partition rootfs {
    image = "rootfs.squashfs"
    size = 64M
  }
}
```

Final SD image becomes:

```
~70–80 MB
```

instead of several hundred MB.

---

# 3. Example Final System Size

Typical optimized output:

```
Kernel Image        6 MB
rootfs.squashfs    32 MB
fip.bin             1 MB
------------------------
Total              ~40 MB
```

SD card image:

```
≈ 70–80 MB
```

Very compact.

---

# 4. Your Final Optimized System

With all improvements:

```
FSBL
OpenSBI
U-Boot
Linux 6.6
BusyBox init
Rust + Slint UI
```

Characteristics:

```
Boot time   ≈ 2–3 s
RAM usage   ≈ 45–55 MB
Disk usage  ≈ 40 MB
SD image    ≈ 70 MB
```

This is **very good for a small RISC-V UI device**.

---

✅ If you'd like, I can also show one more **very useful trick specific to the LicheeRV Nano**:

* how to **develop the Rust + Slint UI on your PC and live-deploy to the board in ~2 seconds**, without rebuilding Buildroot every time. This dramatically speeds up development.
