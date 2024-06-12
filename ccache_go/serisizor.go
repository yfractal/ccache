package main

import (
	"C"
	"bytes"
	"encoding/gob"
	"reflect"
	// "errors"
	"fmt"
	"unsafe"
)

func Encode(data interface{}) ([]byte, error) {
	fmt.Println("3333")
	var buffer bytes.Buffer
	encoder := gob.NewEncoder(&buffer)
	err := encoder.Encode(data)
	if err != nil {
                fmt.Println("error???")
		return nil, err
	}
	fmt.Println("Encode3 bytes!!!!!!....", buffer.Bytes())
	return buffer.Bytes(), nil
}

//export CallEncode
func CallEncode(dataPtr unsafe.Pointer, typePtr unsafe.Pointer, size *C.int) (*C.char) {
	fmt.Printf("dataPtr %v\n", dataPtr)
	fmt.Printf("typePtr %v\n", typePtr)

	fmt.Println("call CallEncode")
	retrievedType := *(*reflect.Type)(typePtr)
	fmt.Println("111")
	val := reflect.NewAt(retrievedType, dataPtr).Elem()
	fmt.Println("111222")
	// Convert reflect.Value to interface{}
	any := val.Interface()
	encodedBytes, err := Encode(any)
	if err != nil {
		fmt.Println("errr!!!!!", any)
		return nil
	}

	fmt.Println("Encoded bytes:", encodedBytes)
	*size = C.int(len(encodedBytes))
	encodedPtr := C.CBytes(encodedBytes)
	return (*C.char)(encodedPtr)
}

// func main() {}