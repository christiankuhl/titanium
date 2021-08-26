#[repr(u8)]
enum FISType {
    // Type of FIS (= Frame Information Structure)
    RegisterH2D = 0x27,   // Register FIS - host to device
    RegisterD2H = 0x34,   // Register FIS - device to host
    DMAActivate = 0x39,   // DMA activate FIS - device to host
    DMASetup = 0x41,      // DMA setup FIS - bidirectional
    Data = 0x46,          // Data FIS - bidirectional
    BISTActivate = 0x58,  // BIST activate FIS - bidirectional
    PIOSetup = 0x5F,      // PIO setup FIS - device to host
    SetDeviceBits = 0xA1, // Set device bits FIS - device to host
}

#[repr(u8)]
enum CommandControl {
    Control = 0,
    Command = 1,
}

#[repr(C, packed)]
struct RegisterH2D {
    fis_type: FISType, // FIS_TYPE_REG_H2D
    pmport: u8,        // Port multiplier
    rsv0: u8,          // Reserved
    c: CommandControl, // 1: Command, 0: Control
    command: u8,       // Command register
    featurel: u8,      // Feature register, 7:0
    lba0: u8,          // LBA low register, 7:0
    lba1: u8,          // LBA mid register, 15:8
    lba2: u8,          // LBA high register, 23:16
    device: u8,        // Device register
    lba3: u8,          // LBA register, 31:24
    lba4: u8,          // LBA register, 39:32
    lba5: u8,          // LBA register, 47:40
    featureh: u8,      // Feature register, 15:8
    countl: u8,        // Count register, 7:0
    counth: u8,        // Count register, 15:8
    icc: u8,           // Isochronous command completion
    control: u8,       // Control register
    rsv1: u64,         // Reserved
}

impl RegisterH2D {
    pub fn new(
        cmdctrl: CommandControl,
        cmdreg: u8,
        ctrlreg: u8,
        device: u8,
        feature: u16,
        lba: u64,
        count: u16,
        icc: u8,
    ) -> Self {
        Self {
            fis_type: FISType::RegisterH2D,
            pmport: 4,
            rsv0: 3,
            c: cmdctrl,
            command: cmdreg,
            featurel: feature as u8,
            lba0: lba as u8,
            lba1: (lba >> 8) as u8,
            lba2: (lba >> 16) as u8,
            device,
            lba3: (lba >> 24) as u8,
            lba4: (lba >> 32) as u8,
            lba5: (lba >> 40) as u8,
            featureh: (feature >> 8) as u8,
            countl: count as u8,
            counth: (count >> 8) as u8,
            icc,
            control: ctrlreg,
            rsv1: 0,
        }
    }
}

// typedef struct tagFIS_REG_D2H
// {
// 	// DWORD 0
// 	uint8_t  fis_type;    // FIS_TYPE_REG_D2H

// 	uint8_t  pmport:4;    // Port multiplier
// 	uint8_t  rsv0:2;      // Reserved
// 	uint8_t  i:1;         // Interrupt bit
// 	uint8_t  rsv1:1;      // Reserved

// 	uint8_t  status;      // Status register
// 	uint8_t  error;       // Error register

// 	// DWORD 1
// 	uint8_t  lba0;        // LBA low register, 7:0
// 	uint8_t  lba1;        // LBA mid register, 15:8
// 	uint8_t  lba2;        // LBA high register, 23:16
// 	uint8_t  device;      // Device register

// 	// DWORD 2
// 	uint8_t  lba3;        // LBA register, 31:24
// 	uint8_t  lba4;        // LBA register, 39:32
// 	uint8_t  lba5;        // LBA register, 47:40
// 	uint8_t  rsv2;        // Reserved

// 	// DWORD 3
// 	uint8_t  countl;      // Count register, 7:0
// 	uint8_t  counth;      // Count register, 15:8
// 	uint8_t  rsv3[2];     // Reserved

// 	// DWORD 4
// 	uint8_t  rsv4[4];     // Reserved
// } FIS_REG_D2H;
