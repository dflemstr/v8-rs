#if defined __cplusplus

#include <v8.h>
#include <v8-platform.h>

typedef v8::ArrayBuffer::Allocator ArrayBuffer_Allocator;
typedef v8::Isolate Isolate;
typedef v8::Platform Platform;
typedef v8::Task Task;
typedef v8::IdleTask IdleTask;

typedef v8::String::Utf8Value String_Utf8Value;

extern "C" {
#else

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef void ArrayBuffer_Allocator;
typedef void Isolate;
typedef void Platform;
typedef void Task;
typedef void IdleTask;
typedef void String_Utf8Value;

#endif /* defined __cplusplus */

struct MaybeBool {
    bool is_set;
    bool value;
};

struct MaybeF64 {
    bool is_set;
    double value;
};

struct MaybeU32 {
    bool is_set;
    uint32_t value;
};

struct MaybeI32 {
    bool is_set;
    int32_t value;
};

struct MaybeU64 {
    bool is_set;
    uint64_t value;
};

struct MaybeI64 {
    bool is_set;
    int64_t value;
};

struct MaybeInt {
    bool is_set;
    int value;
};

struct v8_AllocatorFunctions {
    void *(*Allocate)(size_t length);
    void *(*AllocateUninitialized)(size_t length);
    void (*Free)(void *data, size_t length);
};

enum v8_ExpectedRuntime {
    SHORT_RUNNING_TASK,
    LONG_RUNNING_TASK,
};

struct v8_PlatformFunctions {
    void (*Destroy)();
    size_t (*NumberOfAvailableBackgroundThreads)();
    void (*CallOnBackgroundThread)(Task *task, enum v8_ExpectedRuntime expected_runtime);
    void (*CallOnForegroundThread)(Isolate *isolate, Task *task);
    void (*CallDelayedOnForegroundThread)(Isolate *isolate, Task *task, double delay_in_seconds);
    void (*CallIdleOnForegroundThread)(Isolate *isolate, IdleTask *task);
    bool (*IdleTasksEnabled)(Isolate *isolate);
    double (*MonotonicallyIncreasingTime)();
    // TODO: GetCategoryGroupEnabled
    // TODO: GetCategoryGroupName
    // TODO: AddTraceEvent
    // TODO: UpdateTraceEventDuration
};

Platform *v8_Platform_Create(struct v8_PlatformFunctions platform_functions);
void v8_Platform_Destroy(Platform *platform);

void v8_V8_InitializeICU();
void v8_V8_InitializePlatform(Platform *platform);
void v8_V8_Initialize();
void v8_V8_Dispose();
void v8_V8_ShutdownPlatform();

ArrayBuffer_Allocator *v8_ArrayBuffer_Allocator_Create(struct v8_AllocatorFunctions allocator_functions);
void v8_ArrayBuffer_Allocator_Destroy(ArrayBuffer_Allocator *allocator);

Isolate *v8_Isolate_New(ArrayBuffer_Allocator *allocator);
void v8_Isolate_Dispose(Isolate *isolate);

void v8_Task_Run(Task *task);
void v8_IdleTask_Run(IdleTask *task, double deadline_in_seconds);

#include "v8-glue-generated.h"

Context *v8_Context_New(Isolate *isolate);
void v8_Context_Enter(Isolate *isolate, Context *context);
void v8_Context_Exit(Isolate *isolate, Context *context);

String *v8_String_NewFromUtf8_Normal(Isolate *isolate, const char *data, int length);
String *v8_String_NewFromUtf8_Internalized(Isolate *isolate, const char *data, int length);

int v8_String_WriteUtf8(Isolate *isolate, String *string, char *buffer, int length);

Script *v8_Script_Compile(Isolate *isolate, Context *context, String *source);

Value *v8_Object_CallAsFunction(Isolate *isolate, Object *self, Context *context, Value *recv, int argc, Value *argv[]);

Value *v8_Object_CallAsConstructor(Isolate *isolate, Object *self, Context *context, int argc, Value *argv[]);

#if defined __cplusplus
} /* extern "C" */
#endif /* defined __cplusplus */
