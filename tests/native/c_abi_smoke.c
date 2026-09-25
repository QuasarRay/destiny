#include "destiny_bevy_compat.h"

#include <stdint.h>

int main(void) {
    if (dbc_abi_version() != DBC_ABI_VERSION) return 1;
    DbcBuffer error = {0};
    DbcRuntime *runtime = dbc_runtime_new(NULL, 0, &error);
    dbc_buffer_free(error);
    if (runtime == NULL) return 2;
    const char request[] =
        "{\"title\":\"destiny.Ballpark.__init__\",\"operation\":\"construct\","
        "\"target\":null,\"args\":[false],\"kwargs\":{}}";
    DbcBuffer response = dbc_runtime_call(
        runtime, (const uint8_t *)request, sizeof(request) - 1);
    const int status = response.status;
    dbc_buffer_free(response);
    dbc_runtime_release(runtime);
    return status == 0 ? 0 : 3;
}
