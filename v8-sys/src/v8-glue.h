#if defined __cplusplus

#include <v8.h>
#include <v8-platform.h>

extern "C" {
#else

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#endif /* defined __cplusplus */

struct RustContext;
typedef struct RustContext RustContext;

#define STRUCT_MAYBE_PRIM(PRIM, SUFFIX)         \
    struct Maybe##SUFFIX {                      \
        bool is_set;                            \
        PRIM value;                             \
    };                                          \
    typedef struct Maybe##SUFFIX Maybe##SUFFIX;

STRUCT_MAYBE_PRIM(bool, Bool);
STRUCT_MAYBE_PRIM(unsigned int, UInt);
STRUCT_MAYBE_PRIM(int, Int);
STRUCT_MAYBE_PRIM(unsigned long, ULong);
STRUCT_MAYBE_PRIM(long, Long);
STRUCT_MAYBE_PRIM(double, F64);
STRUCT_MAYBE_PRIM(uint32_t, U32);
STRUCT_MAYBE_PRIM(int32_t, I32);
STRUCT_MAYBE_PRIM(uint64_t, U64);
STRUCT_MAYBE_PRIM(int64_t, I64);

#if defined __cplusplus
typedef v8::ArrayBuffer::Allocator *ArrayBuffer_AllocatorPtr;
#else
typedef struct _ArrayBuffer_Allocator *ArrayBuffer_AllocatorPtr;
#endif /* defined __cplusplus */

#if defined __cplusplus
typedef v8::Isolate *IsolatePtr;
#else
typedef struct _Isolate *IsolatePtr;
#endif /* defined __cplusplus */

#if defined __cplusplus
typedef v8::Platform *PlatformPtr;
#else
typedef struct _Platform *PlatformPtr;
#endif /* defined __cplusplus */

#if defined __cplusplus
typedef v8::Task *TaskPtr;
#else
typedef struct _Task *TaskPtr;
#endif /* defined __cplusplus */

#if defined __cplusplus
typedef v8::IdleTask *IdleTaskPtr;
#else
typedef struct _IdleTask *IdleTaskPtr;
#endif /* defined __cplusplus */

struct v8_AllocatorFunctions {
    void *(*Allocate)(size_t length);
    void *(*AllocateUninitialized)(size_t length);
    void (*Free)(void *data, size_t length);
};
typedef struct v8_AllocatorFunctions v8_AllocatorFunctions;

enum v8_ExpectedRuntime {
    SHORT_RUNNING_TASK,
    LONG_RUNNING_TASK,
};

struct v8_PlatformFunctions {
    void (*Destroy)();
    size_t (*NumberOfAvailableBackgroundThreads)();
    void (*CallOnBackgroundThread)(TaskPtr task, enum v8_ExpectedRuntime expected_runtime);
    void (*CallOnForegroundThread)(IsolatePtr isolate, TaskPtr task);
    void (*CallDelayedOnForegroundThread)(IsolatePtr isolate, TaskPtr task, double delay_in_seconds);
    void (*CallIdleOnForegroundThread)(IsolatePtr isolate, IdleTaskPtr task);
    bool (*IdleTasksEnabled)(IsolatePtr isolate);
    double (*MonotonicallyIncreasingTime)();
    // TODO: GetCategoryGroupEnabled
    // TODO: GetCategoryGroupName
    // TODO: AddTraceEvent
    // TODO: UpdateTraceEventDuration
};
typedef struct v8_PlatformFunctions v8_PlatformFunctions;

PlatformPtr v8_Platform_Create(v8_PlatformFunctions platform_functions);
void v8_Platform_Destroy(PlatformPtr platform);

void v8_V8_InitializeICU();
void v8_V8_InitializePlatform(PlatformPtr platform);
void v8_V8_Initialize();
void v8_V8_Dispose();
void v8_V8_ShutdownPlatform();


ArrayBuffer_AllocatorPtr v8_ArrayBuffer_Allocator_Create(v8_AllocatorFunctions allocator_functions);
void v8_ArrayBuffer_Allocator_Destroy(ArrayBuffer_AllocatorPtr allocator);

IsolatePtr v8_Isolate_New(ArrayBuffer_AllocatorPtr allocator);
void v8_Isolate_SetCaptureStackTraceForUncaughtExceptions_Overview(IsolatePtr self, bool capture, int frame_limit);
void v8_Isolate_SetCaptureStackTraceForUncaughtExceptions_Detailed(IsolatePtr self, bool capture, int frame_limit);
void v8_Isolate_Dispose(IsolatePtr isolate);

void v8_Task_Run(TaskPtr task);
void v8_IdleTask_Run(IdleTaskPtr task, double deadline_in_seconds);

#include "v8-glue-generated.h"

ContextRef v8_Context_New(RustContext c);

StringRef v8_String_NewFromUtf8_Normal(RustContext c, const char *data, int length);
StringRef v8_String_NewFromUtf8_Internalized(RustContext c, const char *data, int length);

int v8_String_WriteUtf8(RustContext c, StringRef string, char *buffer, int length);

ScriptRef v8_Script_Compile(RustContext c, ContextRef context, StringRef source);

ValueRef v8_Object_CallAsFunction(RustContext c, ObjectRef self, ContextRef context, ValueRef recv, int argc, ValueRef argv[]);

ValueRef v8_Object_CallAsConstructor(RustContext c, ObjectRef self, ContextRef context, int argc, ValueRef argv[]);

struct RustContext {
    IsolatePtr isolate;
    ValueRef *exception;
    MessageRef *message;
};

#if defined __cplusplus
} /* extern "C" */
#endif /* defined __cplusplus */
