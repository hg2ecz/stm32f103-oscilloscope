ARCH=arm-none-eabi-
CC=$(ARCH)gcc
CFLAGS = -O2 -Wall -mcpu=cortex-m3 -mthumb -DSTM32F10X_MD
CFLAGS+= -funroll-loops
CFLAGS+= -Icmsis_boot -Iinc -ISTM32_USB-FS-Device_Driver/inc -ISTM32F10x_StdPeriph_Driver/inc/
#LDFLAGS= -s -T STM32F103C8Tx_FLASH.ld
LDFLAGS= -s -T link.ld
# --specs=nosys.specs

OBJS = command.o hw_config.o main.o stm32f10x_it.o usb_desc.o usb_endp.o usb_istr.o usb_prop.o usb_pwr.o
OBJS+= STM32_USB-FS-Device_Driver/src/usb_init.o STM32F10x_StdPeriph_Driver/src/stm32f10x_rcc.o STM32F10x_StdPeriph_Driver/src/stm32f10x_gpio.o
OBJS+= STM32F10x_StdPeriph_Driver/src/stm32f10x_dma.o STM32F10x_StdPeriph_Driver/src/stm32f10x_adc.o STM32F10x_StdPeriph_Driver/src/misc.o
OBJS+= STM32_USB-FS-Device_Driver/src/usb_sil.o STM32_USB-FS-Device_Driver/src/usb_mem.o STM32_USB-FS-Device_Driver/src/usb_regs.o
OBJS+= STM32_USB-FS-Device_Driver/src/usb_core.o STM32_USB-FS-Device_Driver/src/usb_int.o
OBJS+= cmsis_boot/startup/startup_stm32f10x_md.o cmsis_boot/system_stm32f10x.o

TARGET = stm32scope

all: $(OBJS)
	$(ARCH)ld $(OBJS) $(LDFLAGS) -o $(TARGET).elf
	$(ARCH)objcopy -S -O binary $(TARGET).elf $(TARGET).bin
	$(ARCH)size $(TARGET).elf

flash:
	stm32flash -w $(TARGET).bin /dev/ttyUSB0

clean:
	rm -f $(OBJS) $(TARGET).elf $(TARGET).bin
