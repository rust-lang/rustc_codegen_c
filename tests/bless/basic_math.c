// file: basic_math.3cfc46df15d2d47-cgu.0.c
#include <stdint.h>

/* Some helper macros for the generated code */

/** cast from unsigned to signed
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
