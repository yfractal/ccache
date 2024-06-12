package main

/*
#cgo CFLAGS: -I.
#cgo LDFLAGS: -L${SRCDIR}/target/release -lccache_go
#include <stdlib.h>

// int encode(void* p1, void* p2);
void ccache_init();
void ccache_insert(const char *key, void* data_ptr, void* type_ptr);
*/
import "C"
import (
        "unsafe"
        "reflect"
)

type Data struct {
	Name string
	Age  int
}

func main() {
        data := Data {
                Name: "mike",
                Age: 123,
        }

        t := reflect.TypeOf(data)
        typePtr := unsafe.Pointer(&t)
        dataPtr := unsafe.Pointer(&data)

        C.ccache_init();

        key := C.CString("some-key")
        defer C.free(unsafe.Pointer(key))

        C.ccache_insert(key, dataPtr, typePtr)
}
