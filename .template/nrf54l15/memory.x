MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */

  /* Memory layout for nRF54L15 application core (secure), without SoftDevice */
  FLASH : ORIGIN = 0x00000000, LENGTH = 1524K
  RAM : ORIGIN = 0x20000000, LENGTH = 256K

}
