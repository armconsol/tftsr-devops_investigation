// Shim for memset_explicit on MinGW which doesn't provide it
// This is needed for libsodium's secure memory clearing

#if defined(_WIN32) && defined(__MINGW32__)

#include <string.h>

// memset_explicit is available in Windows 8+ but MinGW headers don't always declare it
// Provide a fallback implementation using SecureZeroMemory if available,
// or a volatile memset to prevent compiler optimization
void *memset_explicit(void *s, int c, size_t n) {
    // Try to use Windows API if available
    #ifdef _WIN32_WINNT
        #if _WIN32_WINNT >= 0x0602  // Windows 8+
            extern void *memset_s(void *, size_t, int, size_t);
            return memset_s(s, n, c, n);
        #endif
    #endif

    // Fallback: use volatile to prevent optimization
    volatile unsigned char *p = (volatile unsigned char *)s;
    while (n--) {
        *p++ = (unsigned char)c;
    }
    return s;
}

#endif
