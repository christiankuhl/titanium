/* { dg-do compile } */
/* { dg-options "-O3 -march=z14 -mzarch --save-temps" } */
/* { dg-do run { target { s390_z14_hw } } } */
#include <assert.h>
#include <stdint.h>

__attribute__ ((noipa)) static uint32_t
long_double_to_u32 (long double x)
{
  return x;
}

/* { dg-final { scan-assembler-times {\n\tclfxbr\t} 1 } } */

int
main (void)
{
  assert (long_double_to_u32 (42.L) == 42);
  /* Not (-42 & 0xffffffff) due to loss of precision.  */
  assert (long_double_to_u32 (-42.L) == 0);
}
