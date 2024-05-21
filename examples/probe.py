from bcc import BPF, USDT
import ctypes as ct
import sys

bpf_program = """
#include <uapi/linux/ptrace.h>
#include <linux/sched.h>

struct data_t {
    char method[32];
    char event[16];
    char key[32];
    char trace_id[32];
};

BPF_PERF_OUTPUT(events);

int probe(struct pt_regs *ctx) {
    struct data_t data = {};

    u64 trace_addr = 0;

    bpf_usdt_readarg(1, ctx, &trace_addr);
    bpf_probe_read_user(&data, sizeof(data), (void *)trace_addr);

    events.perf_submit(ctx, &data, sizeof(data));

    return 0;
}
"""

if len(sys.argv) > 1:
    program = sys.argv[1]
else:
    print('Please provide program path, such as \'sudo python3 probe.py /home/ec2-user/ccache/target/release/examples/http_server\'')
    exit()

# Attach to the running process with USDT probes
usdt = USDT(path=program)
usdt.enable_probe(probe="store", fn_name="probe")
# Load and attach BPF program
b = BPF(text=bpf_program, usdt_contexts=[usdt])

# Define output data structure in Python
class Data(ct.Structure):
    _fields_ = [
        ("method", ct.c_char * 32),
        ("event", ct.c_char * 16),
        ("key", ct.c_char * 32),
        ("trace_id", ct.c_char * 32),
    ]

# Callback to handle events
def print_event(cpu, data, size):
    event = ct.cast(data, ct.POINTER(Data)).contents
    print(f"method: {event.method.decode('utf-8', 'replace')}")
    print(f"event: {event.event.decode('utf-8', 'replace')}")
    print(f"key: {event.key.decode('utf-8', 'replace')}")
    print(f"trace_id: {event.trace_id.decode('utf-8', 'replace')}\n")

# Open perf buffer
b["events"].open_perf_buffer(print_event)

# Poll for events
while True:
    try:
        b.perf_buffer_poll()
    except KeyboardInterrupt:
        exit()
