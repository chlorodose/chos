ENTRY(_start)

SECTIONS {
    . = 0xffffffff80000000;
    
    .text : ALIGN(CONSTANT(MAXPAGESIZE)) {
        *(.text*)
    } :text

    .rodata : ALIGN(CONSTANT(MAXPAGESIZE)) {
        *(.rodata*)
    }
    
    .data : ALIGN(CONSTANT(MAXPAGESIZE)) {
        *(.data*)
    }

    .bss : ALIGN(CONSTANT(MAXPAGESIZE)) {
        *(COMMON)
        *(.bss*)
        . = ALIGN(4K);
    }

    KERNEL_END = .;

    .limine_reqs : ALIGN(CONSTANT(MAXPAGESIZE)) {
        KEEP(*(.limine_reqs))
        . = ALIGN(4K);
    }

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note.gnu.*)
    }
}