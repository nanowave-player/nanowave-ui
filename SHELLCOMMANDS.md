# Useful shell commands

```shell
# display off
echo 0 > /sys/class/pwm/pwmchip8/pwm2/enable

# display on
echo 1 > /sys/class/pwm/pwmchip8/pwm2/enable

```



Battery saving questions:
- Turn off onboard LEDs
- Turn off unused GPIO pins
- Reduce CPU frequency ( Governor / Low-Power-CPU modes)