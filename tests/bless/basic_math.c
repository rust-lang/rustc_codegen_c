// file: basic_math.3cfc46df15d2d47-cgu.0.c
#include <stdint.h>

/* Some helper macros for the generated code */

/** Casts an unsigned integer to a signed integer of the same size.
  * This is used to avoid UB when do integer casting in Rust.
  *
  * The parameter `u` is the unsigned type, `s` is the signed type,
  * `v` is the value to cast, and `m` is the maximum value of the signed type.\
  *
  * example: `__rust_utos(uint32_t, int32_t, x, INT32_MAX)`
  */
#define __rust_utos(u, s, v, m) \
    ((v) <= (m) ? ((s)v) : ((s)((u)(v) - (u)(m) - 1)))

int32_t main();
int64_t foo(uint8_t _0, uint16_t _1, uint32_t _2);

int32_t main() { return 0; }

int64_t foo(uint8_t _0, uint16_t _1, uint32_t _2)
{
  int64_t _3 = __rust_utos(uint64_t, int64_t, (int64_t) _0, INT64_MAX);
  return _3;
}
