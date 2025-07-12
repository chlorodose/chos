project('chos', 'rust', meson_version: '1.8.0')

rust = import('rust')

add_project_arguments(
    '-C', 'panic=abort',
    '--target', 'arch' / (host_machine.cpu_family() + '.json'),
    language: 'rust',
)

subproject('std', required: true)

subdir('arch')

libkernel = static_library(
    'kernel',
    'kernel/lib.rs',
    build_by_default: true,
    link_with: [libarch],
)
rust.test(
    'kernel',
    static_library('kernel_test', 'kernel/lib.rs', native: true),
    link_with: libarch_test,
)

subdir('bootloader')