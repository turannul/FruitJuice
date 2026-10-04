obj-m += fruitjuice.o

KDIR ?= /lib/modules/$(shell uname -r)/build
M ?= $(CURDIR)

all:
	$(MAKE) -C $(KDIR) M=$(M) modules

clean:
	$(MAKE) -C $(KDIR) M=$(M) clean
