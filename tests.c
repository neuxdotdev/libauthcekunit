#include "libauthcekunit.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
static int tests_passed = 0;
static int tests_failed = 0;
#define TEST(expr, msg) do { \
    if (expr) { \
        printf("PASS: %s\n", msg); \
        tests_passed++; \
    } else { \
        printf("FAIL: %s\n", msg); \
        tests_failed++; \
    } \
} while(0)
int main() {
    printf("Running libauthcekunit C tests...\n\n");
    libauthcekunit_init_logging();
    const char* html1 = "<input type=\"hidden\" name=\"_token\" value=\"abc123def\">";
    char out[256];
    size_t len = libauthcekunit_extract_token(html1, out, sizeof(out));
    TEST(len > 0 && strcmp(out, "abc123def") == 0, "extract_token basic");
    const char* html2 = "<input name='_token' value='xyz789'>";
    memset(out, 0, sizeof(out));
    len = libauthcekunit_extract_token(html2, out, sizeof(out));
    TEST(len > 0 && strcmp(out, "xyz789") == 0, "extract_token single quotes");
    const char* html3 = "<input type=hidden name=\"_token\" value=\"token-with-dashes_123\" data-test>";
    memset(out, 0, sizeof(out));
    len = libauthcekunit_extract_token(html3, out, sizeof(out));
    TEST(len > 0 && strcmp(out, "token-with-dashes_123") == 0, "extract_token complex attributes");
    const char* html_no = "<input name=\"not_token\" value=\"x\">";
    memset(out, 0, sizeof(out));
    len = libauthcekunit_extract_token(html_no, out, sizeof(out));
    TEST(len == 0, "extract_token no token returns 0");
    const char* html_short = "<input name=\"_token\" value=\"short\">";
    memset(out, 0, sizeof(out));
    len = libauthcekunit_extract_token(html_short, out, sizeof(out));
    TEST(len == 0, "extract_token short token rejected");
    const char* html_bad = "<input name=\"_token\" value=\"bad token\">";
    memset(out, 0, sizeof(out));
    len = libauthcekunit_extract_token(html_bad, out, sizeof(out));
    TEST(len == 0, "extract_token invalid chars rejected");
    const char* html_ok = "<input name=\"_token\" value=\"1234567890\">";
    char small[5];
    len = libauthcekunit_extract_token(html_ok, small, sizeof(small));
    TEST(len == 0, "extract_token buffer too small returns 0");
    const char* email = libauthcekunit_get_email();
    const char* ua = libauthcekunit_get_user_agent();
    TEST(email != NULL, "get_email returns non-NULL");
    TEST(ua != NULL, "get_user_agent returns non-NULL");
    printf("  email: '%s'\n", email);
    printf("  user-agent: '%s'\n", ua);
    const char* test_server = getenv("LIB_CEKUNIT_AUTH_ENV_TEST_SERVER");
    if (test_server && strlen(test_server) > 0) {
        printf("\nReal server testing enabled on: %s\n", test_server);
        const char* test_email = getenv("LIB_CEKUNIT_AUTH_ENV_EMAIL");
        const char* test_password = getenv("LIB_CEKUNIT_AUTH_ENV_PASSWORD");
        if (test_email && test_password && strlen(test_email) > 0 && strlen(test_password) > 0) {
            CookieJarHandle* jar = libauthcekunit_login(test_server);
            TEST(jar != NULL, "login with real server");
            if (jar) {
                int logout_res = libauthcekunit_logout(jar, test_server);
                TEST(logout_res == 0, "logout successful");
            } else {
                printf("  Skipping logout because login failed.\n");
            }
        } else {
            printf("  Login/logout tests skipped (credentials not set).\n");
        }
        char ftoken[256];
        size_t res = libauthcekunit_fetch_token(test_server, ftoken, sizeof(ftoken));
        printf("  fetch_token result: %zu, token: '%s'\n", res, res > 0 ? ftoken : "none");
        int ret = libauthcekunit_fetch_cookies(test_server, "test_cookies.json");
        printf("  fetch_cookies result: %d\n", ret);
    } else {
        printf("\nNo test server URL set, skipping network tests.\n");
    }
    printf("\nTests completed: %d passed, %d failed\n", tests_passed, tests_failed);
    return tests_failed > 0 ? 1 : 0;
}