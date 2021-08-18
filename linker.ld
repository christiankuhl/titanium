/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)
 
/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	. = 1M;
 
	.rodata : ALIGN(4K)
	{
		/* ensure that the multiboot header is at the beginning */
		KEEP(*(.multiboot))
		*(.rodata .rodata.*)
		. = ALIGN(4K);
	}

	.text : ALIGN(4K)
	{
		*(.text .text.*)
		. = ALIGN(4K);
	}
 
	.data.rel.ro : ALIGN(4K) {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
		. = ALIGN(4K);
    }

	.data : ALIGN(4K)
	{
		*(.data .data.*)
		. = ALIGN(4K);
	}

	.bss  : ALIGN(4K)
	{
		*(.bss)
		*(COMMON)
		. = ALIGN(4K);
	}

	.bss.add  : ALIGN(4K)
	{
		*(.bss.*)
		. = ALIGN(4K);
	}

	.eh_frame_hdr : ALIGN(4K) {
		*(.eh_frame_hdr)
		. = ALIGN(4K);
	}

	.eh_frame : ALIGN(4K) {
		*(.eh_frame)
		. = ALIGN(4K);
	}

	.symtab : ALIGN(4K) {
		*(.symtab)
	}

    .shstrtab : ALIGN(4K) {
		*(.shstrtab)
	}

    .strtab : ALIGN(4K) {
		__shstrtab = .;
		*(.strtab)
	}
}

