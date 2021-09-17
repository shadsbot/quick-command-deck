## Quick Command Deck

### Purpose
Act as a quick hub for common actions, not unlike a StreamDeck. The primary differences include being open source, cheap (that is to say, whatever's lying around), and extensible.

### Bill of Materials
- Arduino Nano
- 1602A display
- 3x+ Cherry MX Brown key switches

All of these are arbitrary, and can be swapped out (or in the case of the display, omitted) with minimal effort.

### Project Layout
Currently it's separated into three segments. 
1. [Software](management_app) - Management from the PC's end
2. [Firmware](deck_firmware) - Microcontroller code
3. Physical - Things like schematics and models