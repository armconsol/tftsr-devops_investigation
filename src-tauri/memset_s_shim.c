/* memset_explicit shim for Windows MinGW
 *
 * libsodium-sys-stable expects memset_explicit which isn't available
 * in the MinGW runtime. This provides a compatible implementation.
 */

#include <string.h>

#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

EXPORT void memset_explicit(void *dest, int val, size_t n) {
    volatile unsigned char *p = (volatile unsigned char *)dest;
    while (n--) {
        *p++ = (unsigned char)val;
    }
}
