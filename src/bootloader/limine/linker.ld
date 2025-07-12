ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(1);
    ro PT_LOAD FLAGS(4);
    data PT_LOAD FLAGS(6);
    bl PT_LOAD FLAGS(4);
}

SECTIONS {
    . = 0xffffffff80000000;
    . = ALIGN(CONSTANT(MAXPAGESIZE));
    __KERNEL_TEXT_START = .;
    
    .text : {
        *(.text*)
    } :text


    . = ALIGN(CONSTANT(MAXPAGESIZE));
    __KERNEL_TEXT_RO_BOUNDARY = .;

    .rodata : {
        *(.rodata*)
    } :ro


    . = ALIGN(CONSTANT(MAXPAGESIZE));
    __KERNEL_RO_DATA_BOUNDARY = .;

    .data : {
        *(.data*)
        *(.sdata*)
    } :data

    .bss : {
        *(.bss*)
        *(.sbss*)
        *(COMMON)
    } :data

    .got : {
        *(.got*)
    } :data


    . = ALIGN(CONSTANT(MAXPAGESIZE));
    __KERNEL_DATA_BL_BOUNDARY = .;

    .limine_reqs : {
        KEEP(*(.limine_reqs))
        . = ALIGN(4K);
    } :bl


    . = ALIGN(CONSTANT(MAXPAGESIZE));
    __KERNEL_BL_END = .;
}