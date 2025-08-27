#!/bin/bash

qemu-system-i386 -machine pc -drive file=build/disk.img,format=raw -vnc :2 -gdb tcp::26000 -S &
