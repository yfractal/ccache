package main

/*
#cgo CFLAGS: -I.
#cgo LDFLAGS: -L${SRCDIR}/target/release -lccache_go
#include <stdlib.h>

// int encode(void* p1, void* p2);
void call_greet(void* p1, void* p2);
*/
import "C"
import (
        "fmt"
        "unsafe"
        "reflect"
        // "bytes"
	// "encoding/gob"
)


// Data is a Go struct representing the data you want to encode
type Data struct {
	// Define the fields you want to encode here
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

        fmt.Printf("dataPtr %v\n", dataPtr)
        fmt.Printf("typePtr %v\n", typePtr)
        C.call_greet(dataPtr, typePtr)
        // fmt.Printf("The result is %d\n", result)
}
