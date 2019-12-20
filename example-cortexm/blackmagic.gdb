# Assumes that you have udev rules set up for black magic probe
target extended-remote /dev/ttyBmpGdb

# Enable target power
monitor tpwr enable

# Scan and attach to the first target
monitor swdp_scan
attach 1

# print demangled symbols
set print asm-demangle on

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# *try* to stop at the user entry point (it might be gone due to inlining)
break main

load

# start the process but immediately halt the processor
stepi