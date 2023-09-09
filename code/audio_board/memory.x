MEMORY
{
	ITCM (rwx):  ORIGIN = 0x00000000, LENGTH = 512K
	DTCM (rwx):  ORIGIN = 0x20000000, LENGTH = 512K
	RAM (rwx):   ORIGIN = 0x20200000, LENGTH = 512K
	FLASH (rwx): ORIGIN = 0x60000000, LENGTH = 1984K
}

/* ===--- End imxrt-boot-header.x ---=== */
__flexram_config = 0xFFAAAAAA;
__imxrt_family = 1060;

SECTIONS
{
    .text.code : 
    {
		KEEP(*(.startup))
		*(.flashmem*)
		. = ALIGN(4);
		KEEP(*(.init))
		__preinit_array_start = .;
		KEEP (*(.preinit_array))
		__preinit_array_end = .;
		__init_array_start = .;
		KEEP (*(.init_array))
		__init_array_end = .;
		. = ALIGN(4);
	} > FLASH

     .got : ALIGN(4)
    {
        __global_offset_table_flash_start__ = ADDR(.got) ;
        __global_offset_table_dtc_start__ = LOADADDR(.got) ;
        *(.got* .got.*)
        __global_offset_table_flash_end__ = ABSOLUTE(.) ;
    } >FLASH  

    .text.progmem : 
    {
		*(.progmem*)
		. = ALIGN(4);
	} > FLASH

	.bss ALIGN(4) : {
		*(SORT_BY_ALIGNMENT(SORT_BY_NAME(.bss*)))
		*(SORT_BY_ALIGNMENT(SORT_BY_NAME(COMMON)))
		. = ALIGN(32);
		. = . + 32; /* MPU to trap stack overflow */
	} > DTCM

    .bss.dma : 
    {
		*(.hab_log)
		*(.dmabuffers)
		. = ALIGN(32);
	} > RAM
}

