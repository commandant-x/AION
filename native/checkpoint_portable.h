#ifndef AION_CHECKPOINT_PORTABLE_H
#define AION_CHECKPOINT_PORTABLE_H
#include "aion_sim.h"
/* AION7 canonical checkpoint. Byte order is little-endian, IEEE754 binary64.
   No native padding, pointers, memory dumps or struct ABI dependencies.
   Only ACTIVE nodes and bonds are stored. Re-serialization is deterministic.
   Hash is for accidental corruption, not authentication. */
#define AION7_HEADER_BYTES 32u
#define AION7_FIXED_BYTES 100u
#define AION7_NODE_BYTES 88u
#define AION7_EDGE_BYTES 100u
#define AION7_MAX_BYTES (AION7_HEADER_BYTES+AION7_FIXED_BYTES+AION_MAX_NODES*AION7_NODE_BYTES+AION_MAX_BONDS*AION7_EDGE_BYTES)
unsigned aion7_bytes(const AionSim *s);
unsigned aion7_write(const AionSim *s,unsigned char *out,unsigned capacity);
int aion7_read(AionSim *dst,const unsigned char *input,unsigned length);
#endif
