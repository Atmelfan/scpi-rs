# scpi-rs design document

## What scpi-rs **SHOULD** do
 * Provide a parser/marshall for the SCPI language
 * Make it easy to conform to standards and hard to do wrong
 *

## What scpi-rs **SHALL NOT** do
 * Be tied to a specific platform and/or OS
 * Require `std` or `alloc` (optional alloc/std features are allowed)
 * Require any specific transport/protocol (i.e. `scpi` shall not depend on `vxi-server` and vice-versa)

## Documentation
When implementing a specific standard/protocol code documentation should refer to the
standard it implements. 

Example: A standards command like `*IDN?` should in its doc-comments refer to the
standard chapter where it is defined.

## Architecture
* scpi (IEEE488,SCPI)
* instrument (Device)
    * tcpip
        * instr
        * hislip
        * socket
    * usb
        * tmc
        * serial
    
scpi-tcpip-instr
scpi-tcpip-hislip
scpi-tcpip-socket
scpi-usb-tmc
scpi-usb-serial