Below is a **practical bootstrap script** that sets up a **Buildroot workspace for the LicheeRV Nano** using:

* vendor **FSBL**
* **OpenSBI**
* modern **U-Boot**
* **Buildroot**
* **Linux kernel 6.6**

The script **does not compile everything yet**. Instead it prepares a **clean reproducible environment**, downloads required sources, and creates the files needed to integrate the **vendor FSBL into the final `fip.bin`**.

This approach keeps everything **Buildroot-friendly and reproducible**.

---

# prepare-licheerv-buildroot.sh

```bash
#!/usr/bin/env bash
set -e

# -------- CONFIG --------
WORKDIR=$HOME/licheerv-nano-build
BUILDROOT_VERSION=2024.02
UBOOT_VERSION=v2024.04
KERNEL_VERSION=6.6.32

# Vendor FSBL repository
FSBL_REPO=https://github.com/sipeed/LicheeRV-Nano-Build.git

# ------------------------

echo "Creating workspace at $WORKDIR"
mkdir -p $WORKDIR
cd $WORKDIR

echo "Fetching Buildroot"
git clone https://github.com/buildroot/buildroot.git
cd buildroot
git checkout $BUILDROOT_VERSION
cd ..

echo "Fetching U-Boot"
git clone https://source.denx.de/u-boot/u-boot.git
cd u-boot
git checkout $UBOOT_VERSION
cd ..

echo "Fetching OpenSBI"
git clone https://github.com/riscv-software-src/opensbi.git

echo "Fetching vendor FSBL"
git clone $FSBL_REPO vendor

echo "Creating board directory"
mkdir -p buildroot/board/licheerv-nano

echo "Creating post-image script"

cat << 'EOF' > buildroot/board/licheerv-nano/post-image.sh
#!/usr/bin/env bash
set -e

BINARIES_DIR=$1

echo "Packaging firmware into fip.bin"

FSBL=../vendor/ramdisk/rootfs/public/cv181x/cv181x_bl2.bin
OPENSBI=$BINARIES_DIR/fw_dynamic.bin
UBOOT=$BINARIES_DIR/u-boot.bin

FIPTOOL=../vendor/host-tools/fiptool

$FIPTOOL create \
--bl2 $FSBL \
--fw $OPENSBI \
--nt-fw $UBOOT \
$BINARIES_DIR/fip.bin

echo "fip.bin created"
EOF

chmod +x buildroot/board/licheerv-nano/post-image.sh

echo "Creating Buildroot defconfig"

cat << 'EOF' > buildroot/configs/licheerv_nano_defconfig
BR2_riscv=y
BR2_riscv64=y

BR2_TOOLCHAIN_BUILDROOT_GLIBC=y

BR2_TARGET_GENERIC_HOSTNAME="licheerv"
BR2_TARGET_GENERIC_ISSUE="LicheeRV Nano"

BR2_LINUX_KERNEL=y
BR2_LINUX_KERNEL_CUSTOM_VERSION=y
BR2_LINUX_KERNEL_CUSTOM_VERSION_VALUE="6.6.32"
BR2_LINUX_KERNEL_USE_ARCH_DEFAULT_CONFIG=y

BR2_TARGET_UBOOT=y
BR2_TARGET_UBOOT_BUILD_SYSTEM_KCONFIG=y
BR2_TARGET_UBOOT_BOARD_DEFCONFIG="sipeed_licheerv_nano"

BR2_TARGET_OPENSBI=y
BR2_TARGET_OPENSBI_PLATFORM_GENERIC=y

BR2_PACKAGE_ALSA_LIB=y
BR2_PACKAGE_ALSA_UTILS=y

BR2_PACKAGE_LIBINPUT=y
BR2_PACKAGE_LIBEVDEV=y

BR2_ROOTFS_EXT2=y
BR2_ROOTFS_EXT2_4=y

BR2_ROOTFS_POST_IMAGE_SCRIPT="board/licheerv-nano/post-image.sh"
EOF

echo
echo "Environment prepared."
echo
echo "Next steps:"
echo
echo "cd $WORKDIR/buildroot"
echo "make licheerv_nano_defconfig"
echo "make"
echo
echo "Output images will appear in:"
echo
echo "buildroot/output/images/"
echo
echo "Important artifact:"
echo
echo "fip.bin  (boot firmware with FSBL + OpenSBI + U-Boot)"
```

---

# What the Script Sets Up

Workspace layout:

```
licheerv-nano-build
│
├── buildroot
├── u-boot
├── opensbi
├── vendor
│
└── buildroot/board/licheerv-nano
    └── post-image.sh
```

Buildroot will generate:

```
output/images/
 ├─ Image
 ├─ rootfs.ext4
 ├─ fw_dynamic.bin
 ├─ u-boot.bin
 └─ fip.bin
```

Boot chain:

```
BootROM
 → FSBL (vendor)
 → OpenSBI
 → U-Boot
 → Linux 6.6
 → Buildroot rootfs
```

---

# What You Will Probably Need to Adjust

### FSBL path

Vendor repos change layout often.

You may need to update:

```
cv181x_bl2.bin
```

location.

---

### fiptool

Sometimes it must be built first:

```
make -C vendor/host-tools
```

---

### Kernel defconfig

Later you will likely switch to:

```
sophgo_defconfig
```

or a custom one.

---

# Useful Next Steps

After you confirm boot works:

Add packages:

```
make menuconfig
```

Enable:

* Rust
* Slint dependencies
* WiFi
* Bluetooth

---

If you'd like, I can also show you **three things that will save you a lot of time on this board**:

1. **How to fully automate `fip.bin` generation inside Buildroot** (cleaner integration).
2. **A working kernel 6.6 device tree for the LicheeRV Nano** (this is the part that usually breaks boot).
3. **A minimal framebuffer + libinput + Slint graphics stack** that runs in ~40 MB RAM.
