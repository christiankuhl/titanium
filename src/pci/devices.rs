pub fn description(class_id: u8, subclass_id: u8, _prog_if: u8) -> &'static str {
    match class_id {
        0x00 => {
            "Unclassified";
            match subclass_id {
                0x00 => "Non-VGA-Compatible Unclassified Device",
                0x01 => "VGA-Compatible Unclassified Device",
                _ => "Unspecified",
            }
        },
        0x01 => {
            "Mass Storage Controller";
            match subclass_id {
                0x00 => "SCSI Bus Controller",
                0x01 => "IDE Controller",
                0x02 => "Floppy Disk Controller",
                0x03 => "IPI Bus Controller",
                0x04 => "RAID Controller",
                0x05 => "ATA Controller",
                0x06 => "Serial ATA Controller",
                0x07 => "Serial Attached SCSI Controller",
                0x08 => "Non-Volatile Memory Controller",
                0x80 => "Other Mass Storage Controller",
                _ => "Unspecified",
            }
        },
        0x02 => {
            "Network Controller";
            match subclass_id {
                0x00 => "Ethernet Controller",
                0x01 => "Token Ring Controller",
                0x02 => "FDDI Controller",
                0x03 => "ATM Controller",
                0x04 => "ISDN Controller",
                0x05 => "WorldFip Controller",
                0x06 => "PICMG 2.14 Multi Computing Controller",
                0x07 => "Infiniband Controller",
                0x08 => "Fabric Controller",
                0x80 => "Other Network Controller",
                _ => "Unspecified",
            }
        },
        0x03 => {
            "Display Controller";
            match subclass_id {
                0x00 => "VGA Compatible Controller",
                0x01 => "XGA Controller",
                0x02 => "3D Controller (Not VGA-Compatible)",
                0x80 => "Other Display Controller",
                _ => "Unspecified",
            }
        },
        0x04 => {
            "Multimedia Controller";
            match subclass_id {
                0x00 => "Multimedia Video Controller",
                0x01 => "Multimedia Audio Controller",
                0x02 => "Computer Telephony Device",
                0x03 => "Audio Device",
                0x80 => "Other Multimedia Controller",
                _ => "Unspecified",
            }
        },
        0x05 => {
            "Memory Controller";
            match subclass_id {
                0x00 => "RAM Controller",
                0x01 => "Flash Controller",
                0x80 => "Other Memory Controller",
                _ => "Unspecified",
            }
        },
        0x06 => {
            "Bridge";
            match subclass_id {
                0x00 => "Host Bridge",
                0x01 => "ISA Bridge",
                0x02 => "EISA Bridge",
                0x03 => "MCA Bridge",
                0x04 => "PCI-to-PCI Bridge",
                0x05 => "PCMCIA Bridge",
                0x06 => "NuBus Bridge",
                0x07 => "CardBus Bridge",
                0x08 => "RACEway Bridge",
                0x09 => "PCI-to-PCI Bridge",
                0x0A => "InfiniBand-to-PCI Host Bridge",
                0x80 => "Other Bridge",
                _ => "Unspecified",
            }
        },
        0x07 => {
            "Simple Communication Controller";
            match subclass_id {
                0x00 => "Serial Controller",
                0x01 => "Parallel Controller",
                0x02 => "Multiport Serial Controller",
                0x03 => "Modem",
                0x04 => "IEEE 488.1/2 (GPIB) Controller",
                0x05 => "Smart Card Controller",
                0x80 => "Other Simple Communication Controller",
                _ => "Unspecified",
            }
        },
        0x08 => {
            "Base System Peripheral";
            match subclass_id {
                0x00 => "PIC",
                0x01 => "DMA Controller",
                0x02 => "Timer",
                0x03 => "RTC Controller",
                0x04 => "PCI Hot-Plug Controller",
                0x05 => "SD Host controller",
                0x06 => "IOMMU",
                0x80 => "Other Base System Peripheral",
                _ => "Unspecified",
            }
        },
        0x09 => {
            "Input Device Controller";
            match subclass_id {
                0x00 => "Keyboard Controller",
                0x01 => "Digitizer Pen",
                0x02 => "Mouse Controller",
                0x03 => "Scanner Controller",
                0x04 => "Gameport Controller",
                0x80 => "Other Input Device Controller",
                _ => "Unspecified",
            }
        },
        0x0A => {
            "Docking Station";
            match subclass_id {
                0x00 => "Generic Docking Station",
                0x80 => "Other Docking Station",
                _ => "Unspecified",
            }
        },
        0x0B => {
            "Processor";
            match subclass_id {
                0x00 => "386",
                0x01 => "486",
                0x02 => "Pentium",
                0x03 => "Pentium Pro",
                0x10 => "Alpha",
                0x20 => "PowerPC",
                0x30 => "MIPS",
                0x40 => "Co-Processor",
                0x80 => "Other Processor",
                _ => "Unspecified",
            }
        },
        0x0C => {
            "Serial Bus Controller";
            match subclass_id {
                0x00 => "FireWire (IEEE 1394) Controller",
                0x01 => "ACCESS Bus Controller",
                0x02 => "SSA",
                0x03 => "USB Controller",
                0x04 => "Fibre Channel",
                0x05 => "SMBus Controller",
                0x06 => "InfiniBand Controller",
                0x07 => "IPMI Interface",
                0x08 => "SERCOS Interface (IEC 61491)",
                0x09 => "CANbus Controller",
                0x80 => "Other Serial Bus Controller",
                _ => "Unspecified",
            }
        },
        0x0D => {
            "Wireless Controller";
            match subclass_id {
                0x00 => "iRDA Compatible Controller",
                0x01 => "Consumer IR Controller",
                0x10 => "RF Controller",
                0x11 => "Bluetooth Controller",
                0x12 => "Broadband Controller",
                0x20 => "Ethernet Controller (802.1a)",
                0x21 => "Ethernet Controller (802.1b)",
                0x80 => "Other Wireless Controller",
                _ => "Unspecified",
            }
        },
        0x0E => {
            "Intelligent Controller";
            match subclass_id {
                0x00 => "I20",
                _ => "Unspecified",
            }
        },
        0x0F => {
            "Satellite Communication Controller";
            match subclass_id {
                0x01 => "Satellite TV Controller",
                0x02 => "Satellite Audio Controller",
                0x03 => "Satellite Voice Controller",
                0x04 => "Satellite Data Controller",
                _ => "Unspecified",
            }
        },
        0x10 => {
            "Encryption Controller";
            match subclass_id {
                0x00 => "Network and Computing Encrpytion/Decryption",
                0x10 => "Entertainment Encryption/Decryption",
                0x80 => "Other Encryption Controller",
                _ => "Unspecified",
            }
        },
        0x11 => {
            "Signal Processing Controller";
            match subclass_id {
                0x00 => "DPIO Modules",
                0x01 => "Performance Counters",
                0x10 => "Communication Synchronizer",
                0x20 => "Signal Processing Management",
                0x80 => "Other Signal Processing Controller",
                _ => "Unspecified",
            }
        },
        0x12 => {
            "Processing Accelerator"
        },
        0x13 => {
            "Non-Essential Instrumentation"
        },
        0x40 => {
            "Co-Processor"
        },
        0xFF => {
            "Unassigned Class (Vendor specific)"
        },
        _ => "Reserved"
    }
}