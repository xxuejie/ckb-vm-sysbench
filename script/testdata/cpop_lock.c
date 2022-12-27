/*  Script Description:
 *  - Args should be two little endian unsigned integer: num0 and num1.
 *  - Returns CKB_SUCCESS if and only if any follow conditions satisfied.
 *    - `num0 == 0 && num1 == 0`
 *    - `cpop(num0) == num 1`
 */

#include "ckb_syscalls.h"
#include "blockchain.h"

#ifdef DEBUG
#include <stdio.h>
#else
#define ckb_debug(...)
#define sprintf(...)
#endif

#define SCRIPT_SIZE 32768

static uint64_t cpop (uint64_t rs1) {
    uint64_t rd;
    asm volatile (
        "mv s2, %1\n"
        // "cpop s2, s2\n"
        ".byte 0x13,0x19,0x29,0x60\n"
        "mv %0, s2\n"
        : "=r"(rd)
        : "r"(rs1)
        : "s2"
    );
    return rd;
}

uint64_t read_u64_le (const uint8_t *src) {
    return *(const uint64_t *)src;
}

int main (int argc, char *argv[]) {
    int ret;
    uint64_t len = SCRIPT_SIZE;
    uint8_t script[SCRIPT_SIZE];
#ifdef DEBUG
    char message[2048];
#endif

    ret = ckb_load_script(script, &len, 0);
    if (ret != CKB_SUCCESS) {
        return -1;
    }
    if (len > SCRIPT_SIZE) {
        return -2;
    }

    mol_seg_t script_seg;
    mol_seg_t args_seg;
    mol_seg_t bytes_seg;
    script_seg.ptr = (uint8_t *)script;
    script_seg.size = len;
    if (MolReader_Script_verify(&script_seg, false) != MOL_OK) {
        return -3;
    }
    args_seg = MolReader_Script_get_args(&script_seg);
    bytes_seg = MolReader_Bytes_raw_bytes(&args_seg);

    if (bytes_seg.size != 8 * 2) {
        return -4;
    }

    volatile uint64_t num0 = read_u64_le(bytes_seg.ptr);
    volatile uint64_t num1 = read_u64_le(bytes_seg.ptr+8);

    sprintf(message, "num0 = %ld", num0); ckb_debug(message);
    sprintf(message, "num1 = %ld", num1); ckb_debug(message);

    if (num0 == 0 && num1 == 0) {
        return CKB_SUCCESS;
    }

    volatile uint64_t num1_actual = cpop(num0);
    sprintf(message, "cpop(%lx) = %ld (actual) == %ld (expected)", num0, num1_actual, num1); ckb_debug(message);

    if (num1 != num1_actual) {
        return -5;
    }

    return CKB_SUCCESS;
}
