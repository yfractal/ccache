package main

import (
	"C"
	"bytes"
	"encoding/gob"
	"reflect"
	"fmt"
	"unsafe"
)

func decodeHelper(b []byte, t reflect.Type) (interface{}, error) {
	decoded := createInstance(t)
	buffer := bytes.NewBuffer(b)
	decoder := gob.NewDecoder(buffer)

	err := decoder.Decode(decoded)

	if err != nil {
		fmt.Println("Error decoding:", err)
		return nil, err
	}

	return decoded, nil
}

func createInstance(t reflect.Type) interface{} {
	return reflect.New(t).Interface()
}

func encodeHelper(data interface{}) ([]byte, error) {
	var buffer bytes.Buffer
	encoder := gob.NewEncoder(&buffer)
	err := encoder.Encode(data)
	if err != nil {
        fmt.Println("error: ", err)
		return nil, err
	}
	fmt.Println("encodeHelper bytes", buffer.Bytes())
	return buffer.Bytes(), nil
}


func byteSliceToCChar(b []byte) *C.char {
	return (*C.char)(C.CBytes(b))
}

func cCharToByteSlice(cstr *C.char, length C.int) []byte {
	return C.GoBytes(unsafe.Pointer(cstr), length)
}

//export Encode
func Encode(dataPtr unsafe.Pointer, typePtr unsafe.Pointer, size *C.int) (*C.char) {
	fmt.Printf("dataPtr %v\n", dataPtr)
	fmt.Printf("typePtr %v\n", typePtr)

	retrievedType := *(*reflect.Type)(typePtr)
	val := reflect.NewAt(retrievedType, dataPtr).Elem()
	any := val.Interface()
	encodedBytes, err := encodeHelper(any)

	if err != nil {
		fmt.Println("errr: ", err)
		return nil
	}

	*size = C.int(len(encodedBytes))
	return byteSliceToCChar(encodedBytes)
}

//export Decode
func Decode(bytes *C.char, typePtr unsafe.Pointer, length C.int) (dataPtr unsafe.Pointer) {
	byteSlice := cCharToByteSlice(bytes, length)

	t := *(*reflect.Type)(typePtr)
	data, _ := decodeHelper(byteSlice, t)

	return unsafe.Pointer(&data)
}