#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct CookieJarHandle CookieJarHandle;

void libauthcekunit_init_logging(void);

struct CookieJarHandle *libauthcekunit_login(const char *base_url);

int32_t libauthcekunit_logout(struct CookieJarHandle *handle, const char *base_url);

void libauthcekunit_free_jar(struct CookieJarHandle *handle);

uintptr_t libauthcekunit_fetch_token(const char *url, char *out, uintptr_t out_size);

int32_t libauthcekunit_fetch_cookies(const char *url, const char *file_path);

uintptr_t libauthcekunit_extract_token(const char *html, char *out, uintptr_t out_size);

const char *libauthcekunit_get_email(void);

const char *libauthcekunit_get_user_agent(void);
