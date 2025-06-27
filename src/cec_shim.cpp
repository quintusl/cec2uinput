#include <iostream>
#include <libcec/cec.h>
#include <queue>
#include <mutex>

using namespace CEC;

ICECAdapter* g_cec_adapter = nullptr;
std::queue<cec_command> cec_message_queue;
std::mutex cec_message_mutex;

class MyCECCallbacks : public ICECCallbacks
{
public:
    void ReceivedCommand(const cec_command* command)
    {
        std::lock_guard<std::mutex> lock(cec_message_mutex);
        cec_message_queue.push(std::move(*command));
    }
};

MyCECCallbacks my_cec_callbacks;

extern "C" void initialize_cec() {
    libcec_configuration config;
    config.Clear();

    snprintf(config.strDeviceName, sizeof(config.strDeviceName), "cec2uinput");
    config.clientVersion       = LIBCEC_VERSION_CURRENT;
    config.bActivateSource     = 0;
    config.callbacks           = &my_cec_callbacks;
    config.deviceTypes.Add(CEC_DEVICE_TYPE_RECORDING_DEVICE);

    g_cec_adapter = CECInitialise(&config);
    if (!g_cec_adapter) {
        std::cerr << "Failed to initialize CEC adapter" << std::endl;
        return;
    }

    // Open the CEC adapter (assuming default port or auto-detection)
    if (!g_cec_adapter->Open("", 1000)) { // Empty string for auto-detection, 1000ms timeout
        std::cerr << "Failed to open CEC adapter" << std::endl;
        // LibCEC::DestroyCECAdapter(g_cec_adapter); // TODO: Find correct way to destroy adapter
        g_cec_adapter = nullptr;
        return;
    }
}

// Define a C struct to hold CEC message data
typedef struct {
    unsigned char opcode;
    unsigned char num_params;
    unsigned char params[16];
} CecMessage;

extern "C" int get_cec_message(CecMessage* msg) {
    std::lock_guard<std::mutex> lock(cec_message_mutex);
    if (cec_message_queue.empty()) {
        return 0; // No message received
    }

    const cec_command& cec_cmd = cec_message_queue.front();
    msg->opcode = cec_cmd.opcode;
    msg->num_params = cec_cmd.parameters.size;
    for (int i = 0; i < msg->num_params; ++i) {
        msg->params[i] = cec_cmd.parameters.data[i];
    }
    cec_message_queue.pop();
    return 1; // Message received
}
