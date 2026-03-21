# Explanation: 01-bootloader

At power-on, firmware performs hardware initialization and eventually asks a boot device for the first sector. In BIOS mode, that sector is loaded to memory at `0x7c00` and execution jumps there. Your boot sector is therefore both data and executable code with a strict size budget: exactly 512 bytes.

The boot signature (`0x55 0xaa`) is a firmware sanity check. If those bytes are missing at offsets 510 and 511, BIOS usually treats the sector as non-bootable and moves on. This is why padding matters: your code and data must end before byte 510 so the signature remains in the required location.

Printing text in this milestone typically uses BIOS interrupt services because no operating system exists yet. The common path is teletype output through `int 0x10` with `ah=0x0e`, sending one character at a time until a null terminator.

This milestone gives you three foundational constraints that repeat throughout OS work:

- hard binary layout requirements
- direct hardware/firmware interfaces
- verification by observable behavior, not assumptions

Once this passes, you have a reliable execution foothold for subsequent kernel milestones.
