# Connect to gdb remote server
target extended-remote :3333

# Configure TPIU/SWO for ITM trace output on STM32F3 Discovery
# Set SWO pin speed (SWO clock = HCLK / (divisor + 1))
# For STM32F3 @ 72 MHz, divisor=7 gives 9 MHz SWO clock
#monitor tpiu config internal -o /tmp/itm.log uart off 72000000 9000000
monitor tpiu config internal itm.log uart off 8000000

# Enable ITM stimulus port 0 (for iprintln! output) <reason for $itm.stim[0] >
monitor itm port 0 on

# Load will flash the code
load

# Enable demangling asm names on disassembly
set print asm-demangle on

# Enable pretty printing
set print pretty on

# Disable style sources as the default colors can be hard to read
set style sources off

# Set a breakpoint at main, aka entry
break main

# Set a breakpoint at DefaultHandler
break DefaultHandler

# Set a breakpoint at HardFault
break HardFault

# Continue running until we hit the main breakpoint
continue

# Step from the trampoline code in entry into main
step