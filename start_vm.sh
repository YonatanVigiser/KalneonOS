#!/bin/bash

qemu-system-i386 -machine pc -drive file=build/kalneonos-x86-legacy.img,format=raw -vnc :2 -gdb tcp::26000 -S &
