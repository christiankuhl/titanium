/* { dg-require-effective-target arm_v8_1m_mve_fp_ok } */
/* { dg-add-options arm_v8_1m_mve_fp } */
/* { dg-additional-options "-O2" } */

#include "arm_mve.h"
float16x8_t
foo1 (float16x8_t a, float16x8_t b, float16_t c, mve_pred16_t p)
{
  return vfmaq_m (a, b, 23.23, p);
}

/* { dg-final { scan-assembler-not "__ARM_undef" } } */
