MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */

  /* Memory layout for nRF54L15 application core (secure), without SoftDevice */
  /* Keep two 4 KiB erase pages at the end of RRAM separate from the firmware image. */
  FLASH : ORIGIN = 0x00000000, LENGTH = 1516K
  STORAGE : ORIGIN = 0x0017B000, LENGTH = 8K
  RAM : ORIGIN = 0x20000000, LENGTH = 256K

}

__storage_start = ORIGIN(STORAGE);
__storage_end = ORIGIN(STORAGE) + LENGTH(STORAGE);
