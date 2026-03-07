To make **GPIO work on the LicheeRV Nano with a mainline 6.6 kernel**, the key work is **device-tree support and enabling the correct pinctrl/GPIO driver for the SG2002/CV181x family**. The hardware itself is simple; the challenge is that the vendor kernel (5.10) uses slightly different DTS and sometimes out-of-tree pinctrl code.

The SoC used on the Nano is from **Sophgo** (formerly Cvitek). Support for this platform has gradually appeared upstream in **Linux kernel**, but vendor device trees still contain details you will likely need to port.

Below is the practical checklist.

---

# 1. Ensure the Correct GPIO Driver Is Enabled

Kernel 6.6 already contains the **CVITEK/Sophgo GPIO controller driver**.

Enable in your kernel config:

```text
CONFIG_GPIOLIB=y
CONFIG_GPIO_CDEV=y
CONFIG_PINCTRL=y
CONFIG_PINCTRL_SOPHGO=y
CONFIG_GPIO_SOPHGO=y
```

Also enable sysfs or character device GPIO access:

```text
CONFIG_GPIO_SYSFS=y
```

(or use the modern `gpiod` interface).

---

# 2. Device Tree Is the Most Important Part

GPIO will not work unless the **device tree defines the GPIO banks correctly**.

Typical nodes look like this (simplified):

```dts
gpio0: gpio@03020000 {
    compatible = "sophgo,cv1812-gpio";
    reg = <0x03020000 0x1000>;
    gpio-controller;
    #gpio-cells = <2>;
    interrupts = <25>;
};
```

Some boards expose **multiple GPIO banks**:

```dts
gpio1: gpio@03021000 { ... };
gpio2: gpio@03022000 { ... };
```

These must match the **vendor DTS**.

Your best strategy:

1. Extract DTS from the vendor kernel.
2. Compare it with upstream DTS.
3. Port missing nodes.

You can extract it with:

```bash
dtc -I dtb -O dts -o vendor.dts vendor.dtb
```

---

# 3. Pinmux Configuration

Pins default to alternate functions unless configured as GPIO.

Example pinctrl definition:

```dts
pinctrl {
    gpio0_pins: gpio0-pins {
        pins = "PINMUX_GPIO0";
        function = "gpio";
    };
};
```

Then enable it for your device:

```dts
&pinctrl {
    pinctrl-names = "default";
    pinctrl-0 = <&gpio0_pins>;
};
```

Without correct **pinmux settings**, the pin will not behave as GPIO.

---

# 4. Enable the GPIO Character Interface

Modern kernels use `libgpiod`.

Install tools in **Buildroot**:

```text
BR2_PACKAGE_LIBGPIOD=y
BR2_PACKAGE_LIBGPIOD_TOOLS=y
```

Test GPIO:

```bash
gpiodetect
gpioinfo
gpioset gpiochip0 3=1
gpioget gpiochip0 3
```

---

# 5. Verify the GPIO Controller Appears

After boot:

```bash
dmesg | grep gpio
```

Expected:

```text
gpiochip0: registered GPIOs 0-31
gpiochip1: registered GPIOs 32-63
```

Then:

```bash
gpiodetect
```

Example:

```text
gpiochip0 [cv1812-gpio] (32 lines)
```

If nothing appears, the problem is **device tree or driver config**.

---

# 6. Common Problems When Moving to Kernel 6.6

### Missing pinctrl node

Vendor kernels sometimes place pinctrl definitions in different files.

### Different compatible strings

Vendor example:

```dts
compatible = "cvitek,cv1812-gpio";
```

Upstream example:

```dts
compatible = "sophgo,cv1812-gpio";
```

You may need to update these.

---

### Clock dependencies

Some SoC GPIO blocks require a clock.

Example:

```dts
clocks = <&clk GPIO_CLK>;
```

If missing, the driver may probe but not function.

---

# 7. Testing GPIO Quickly

Export a pin via libgpiod:

```bash
gpioset gpiochip0 5=1
```

Check with a multimeter or LED.

For input:

```bash
gpioget gpiochip0 5
```

---

# 8. Typical GPIO Number Mapping

GPIO numbering usually follows:

```text
bank * 32 + pin
```

Example:

```text
GPIO1_5 = 37
```

But with `libgpiod` you should use **line names instead of global numbers**.

---

# 9. Minimal Kernel Configuration Summary

Your minimal config should include:

```text
CONFIG_GPIOLIB
CONFIG_GPIO_SOPHGO
CONFIG_PINCTRL_SOPHGO
CONFIG_GPIO_CDEV
CONFIG_OF_GPIO
```

That is usually enough.

---

# 10. Quick Sanity Check

After boot you should see:

```text
/dev/gpiochip0
/dev/gpiochip1
```

If these exist, GPIO is working.

---

✅ **Bottom line**

For kernel 6.6 the steps are:

1. Enable the Sophgo GPIO + pinctrl drivers
2. Port the vendor **device tree GPIO nodes**
3. Ensure correct pinmux configuration
4. Use **libgpiod** to access pins

No vendor kernel patches are normally required — only DTS adjustments.

---

If you'd like, I can also show you **the exact GPIO pin mapping of the LicheeRV Nano headers**, which is extremely useful when writing device-tree overlays or controlling LEDs/buttons.
