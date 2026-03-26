
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 5
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_4_end

    .global _app_names
_app_names:
    .string "ch2_exit"
    .string "ch2_hello_world"
    .string "ch2_power"
    .string "ch2_write1"
    .string "ch2t_write0"

    .section .data
    .global app_0_start
    .global app_0_end
    .align 3
app_0_start:
    .incbin "../rcore_tutorial_tests/user/build/bin/ch2_exit.bin"
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
    .align 3
app_1_start:
    .incbin "../rcore_tutorial_tests/user/build/bin/ch2_hello_world.bin"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
    .align 3
app_2_start:
    .incbin "../rcore_tutorial_tests/user/build/bin/ch2_power.bin"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
    .align 3
app_3_start:
    .incbin "../rcore_tutorial_tests/user/build/bin/ch2_write1.bin"
app_3_end:

    .section .data
    .global app_4_start
    .global app_4_end
    .align 3
app_4_start:
    .incbin "../rcore_tutorial_tests/user/build/bin/ch2t_write0.bin"
app_4_end:
