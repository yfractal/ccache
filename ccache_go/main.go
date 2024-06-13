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
        "fmt"
)

type Data struct {
	Name string
	Age  int
}

func pointerToStruct(ptr unsafe.Pointer) interface{} {
	return *(*interface{})(ptr)
}

func main() {
        data := Data {
                Name: "mike",
                Age: 123,
        }

        C.ccache_init();

        key := C.CString("some-key")
        defer C.free(unsafe.Pointer(key))

        t := reflect.TypeOf(data)
        typePtr := unsafe.Pointer(&t)
        dataPtr := unsafe.Pointer(&data)

        var l C.int
        encoded := Encode(dataPtr, typePtr, &l)
        decodedPtr := Decode(encoded, unsafe.Pointer(&t), 48)
        fmt.Println("decodedData",  pointerToStruct(decodedPtr))
}
