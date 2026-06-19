/* memset_explicit shim for Windows MinGW
 *
 * libsodium-sys-stable expects memset_explicit which isn't available
 * in the MinGW runtime. This provides a compatible implementation.
 *
 * This shim is compiled as part of the build process when targeting
 * Windows with MinGW (x86_64-w64-mingw32).
 */

#include <string.h>
#include <stddef.h>

/* 
 * Windows symbol export macros.
 * __declspec(dllexport) ensures the symbol is visible in the compiled object.
 */
#if defined(_WIN32) || defined(__WIN32__) || defined(__MINGW32__)
    #define EXPORT __declspec(dllexport)
#else
    #define EXPORT __attribute__((visibility("default")))
#endif

/*
 * memset_explicit implementation - a secure memory clearing function.
 * Uses volatile to prevent compiler optimization from removing the memset.
 * This matches the signature expected by libsodium.
 */
EXPORT void memset_explicit(void *dest, int val, size_t n) {
    volatile unsigned char *p = (volatile unsigned char *)dest;
    while (n--) {
        *p++ = (unsigned char)val;
    }
}
