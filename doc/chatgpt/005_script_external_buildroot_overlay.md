Yes — using a **Buildroot external tree** is the *recommended approach* for maintainability. It keeps all board-specific logic outside the main **Buildroot** tree and makes upgrades much easier.

Below is a **complete bootstrap script** that:

1. Creates a workspace
2. Clones **U-Boot**, **OpenSBI**, and vendor FSBL
3. Creates a **`buildroot-external-licheervnano`** tree
4. Adds a **defconfig + board files**
5. Runs the `make` commands to build the image
6. Produces `fip.bin` with FSBL + OpenSBI + U-Boot + Linux 6.6

It should give you a **fully reproducible Buildroot environment**.

---

# prepare-licheerv-nano.sh

```bash
#!/usr/bin/env bash
set -e

WORKDIR=$HOME/licheervnano
BUILDROOT_VERSION=2024.02
UBOOT_VERSION=v2024.04
KERNEL_VERSION=6.6.32

echo "Creating workspace"
mkdir -p $WORKDIR
cd $WORKDIR

echo "Cloning Buildroot"
git clone https://github.com/buildroot/buildroot.git
cd buildroot
git checkout $BUILDROOT_VERSION
cd ..

echo "Cloning U-Boot"
git clone https://source.denx.de/u-boot/u-boot.git
cd u-boot
git checkout $UBOOT_VERSION
cd ..

echo "Cloning OpenSBI"
git clone https://github.com/riscv-software-src/opensbi.git

echo "Cloning vendor SDK (for FSBL + fiptool)"
git clone https://github.com/sipeed/LicheeRV-Nano-Build.git vendor

echo "Creating external Buildroot tree"

mkdir -p buildroot/buildroot-external-licheervnano

EXT=$WORKDIR/buildroot/buildroot-external-licheervnano

mkdir -p $EXT/board/licheervnano
mkdir -p $EXT/configs

cat << 'EOF' > $EXT/external.desc
name: LicheeRVNano
desc: External tree for LicheeRV Nano
EOF

cat << 'EOF' > $EXT/external.mk
include $(sort $(wildcard $(BR2_EXTERNAL_LICHEERVNANO_PATH)/board/*/*.mk))
EOF

echo "Creating board.mk"

cat << 'EOF' > $EXT/board/licheervnano/board.mk
LICHEERV_POST_IMAGE_SCRIPT = $(BR2_EXTERNAL_LICHEERVNANO_PATH)/board/licheervnano/post-image.sh
EOF

echo "Creating post-image script"

cat << 'EOF' > $EXT/board/licheervnano/post-image.sh
#!/usr/bin/env bash
set -e

BINARIES_DIR=$1

FSBL=../../vendor/ramdisk/rootfs/public/cv181x/cv181x_bl2.bin
FIPTOOL=../../vendor/host-tools/fiptool

OPENSBI=$BINARIES_DIR/fw_dynamic.bin
UBOOT=$BINARIES_DIR/u-boot.bin

echo "Creating fip.bin"

$FIPTOOL create \
  --bl2 $FSBL \
  --fw $OPENSBI \
  --nt-fw $UBOOT \
  $BINARIES_DIR/fip.bin

echo "fip.bin ready"
EOF

chmod +x $EXT/board/licheervnano/post-image.sh

echo "Creating Buildroot defconfig"

cat << EOF > $EXT/configs/licheervnano_defconfig
BR2_riscv=y
BR2_riscv64=y

BR2_TOOLCHAIN_BUILDROOT_GLIBC=y

BR2_TARGET_GENERIC_HOSTNAME="licheervnano"

BR2_LINUX_KERNEL=y
BR2_LINUX_KERNEL_CUSTOM_VERSION=y
BR2_LINUX_KERNEL_CUSTOM_VERSION_VALUE="$KERNEL_VERSION"
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

BR2_ROOTFS_POST_IMAGE_SCRIPT="\$(BR2_EXTERNAL_LICHEERVNANO_PATH)/board/licheervnano/post-image.sh"
EOF

echo "Preparing build"

cd buildroot

make BR2_EXTERNAL=$EXT licheervnano_defconfig

echo "Starting build (this will take a while)"

make BR2_EXTERNAL=$EXT

echo
echo "Build complete"
echo
echo "Artifacts:"
echo
echo "buildroot/output/images/"
echo
echo "Important files:"
echo "  fip.bin"
echo "  Image"
echo "  rootfs.ext4"
```

---

# Resulting Directory Layout

```text
licheervnano/
│
├── buildroot
│   └── buildroot-external-licheervnano
│
├── opensbi
├── u-boot
├── vendor
│
└── buildroot/output/images
    ├── Image
    ├── rootfs.ext4
    ├── fw_dynamic.bin
    ├── u-boot.bin
    └── fip.bin
```

---

# Build Process

The script automatically runs:

```bash
make BR2_EXTERNAL=buildroot-external-licheervnano licheervnano_defconfig
make BR2_EXTERNAL=buildroot-external-licheervnano
```

---

# Boot Files on SD Card

Typical layout:

```text
FAT partition
  fip.bin

EXT4 partition
  rootfs
```

Boot chain:

```
BootROM
 → FSBL
 → OpenSBI
 → U-Boot
 → Linux 6.6
```

---

# Why the External Tree is Better

Advantages:

* clean upgrades of **Buildroot**
* board support isolated
* easier CI integration
* easier patch management

---

✅ If you'd like, I can also show you **three improvements that make this setup much more robust**:

1. **Automatic kernel + U-Boot patch management inside the external tree**
2. **Fully reproducible `fip.bin` generation without relying on the vendor SDK**
3. **Automatic SD-card image generation (`sdcard.img`) so flashing becomes a single `dd` command**.
