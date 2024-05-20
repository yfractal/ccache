from bcc import BPF, USDT
import ctypes as ct

bpf_program = """
#include <uapi/linux/ptrace.h>
#include <linux/sched.h>

struct data_t {
    char method[10];
    char event[4];
    char key[64];
    char trace_id[64];
    u64 ts;
};

BPF_PERF_OUTPUT(events);

int probe(struct pt_regs *ctx) {
    struct data_t data = {};

    u64 method_addr = 0;
    u64 event_addr = 0;
    u64 key_addr = 0;
    u64 trace_id_addr = 0;

    bpf_usdt_readarg(1, ctx, &method_addr);
    bpf_probe_read_user(&data.method, sizeof(data.method), (void *)method_addr);

    bpf_usdt_readarg(2, ctx, &event_addr);
    bpf_probe_read_user(&data.event, sizeof(data.event), (void *)event_addr);

    bpf_usdt_readarg(3, ctx, &key_addr);
    bpf_probe_read_user(&data.key, sizeof(data.key), (void *)key_addr);

    bpf_usdt_readarg(4, ctx, &trace_id_addr);
    bpf_probe_read_user(&data.trace_id, sizeof(data.trace_id), (void *)trace_id_addr);

    data.ts = bpf_ktime_get_ns();

    events.perf_submit(ctx, &data, sizeof(data));

    return 0;
}
"""

# Attach to the running process with USDT probes
usdt = USDT(path="/home/ec2-user/ccache/target/release/examples/http_server")
usdt.enable_probe(probe="store", fn_name="probe")
# Load and attach BPF program
b = BPF(text=bpf_program, usdt_contexts=[usdt])

# Define output data structure in Python
class Data(ct.Structure):
    _fields_ = [
        ("method", ct.c_char * 10),
        ("event", ct.c_char * 5),
        ("key", ct.c_char * 64),
        ("trace_id", ct.c_char * 64),
        ("ts", ct.c_ulonglong)
    ]

# Callback to handle events
def print_event(cpu, data, size):
    event = ct.cast(data, ct.POINTER(Data)).contents
    print(f"method: {event.method.decode('utf-8', 'replace')}")
    print(f"event: {event.event.decode('utf-8', 'replace')}")
    print(f"key: {event.key.decode('utf-8', 'replace')}")
    print(f"trace_id: {event.trace_id.decode('utf-8', 'replace')}")
    print(f"ts: {event.ts}")

# Open perf buffer
b["events"].open_perf_buffer(print_event)

# Poll for events
while True:
    try:
        b.perf_buffer_poll()
    except KeyboardInterrupt:
        exit()
