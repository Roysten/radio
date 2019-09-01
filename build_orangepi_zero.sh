#!/bin/sh

 CFLAGS="-mfpu=vfpv4" CC=armv7-linux-musleabihf-gcc cargo build --target armv7-unknown-linux-musleabihf $@
