#if defined __cplusplus

#include <v8.h>
#include <v8-platform.h>

extern "C" {
#else

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#endif /* defined __cplusplus */

/* A context passed in from Rust that handles isolation and exception
   handling.
 */
struct RustContext;
typedef struct RustContext RustContext;

/* Structs for "maybes" containing primitives */
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

/* Very special classes that we don't want to auto-map. */
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

/* Special structs simulating vtables */
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
};
typedef struct v8_PlatformFunctions v8_PlatformFunctions;

/* Copy-paste of enums */

enum AccessControl {
    AccessControl_DEFAULT               = 0,
    AccessControl_ALL_CAN_READ          = 1,
    AccessControl_ALL_CAN_WRITE         = 1 << 1,
    AccessControl_PROHIBITS_OVERWRITING = 1 << 2
};
typedef enum AccessControl AccessControl;

enum PropertyFilter {
    PropertyFilter_ALL_PROPERTIES = 0,
    PropertyFilter_ONLY_WRITABLE = 1,
    PropertyFilter_ONLY_ENUMERABLE = 2,
    PropertyFilter_ONLY_CONFIGURABLE = 4,
    PropertyFilter_SKIP_STRINGS = 8,
    PropertyFilter_SKIP_SYMBOLS = 16
};
typedef enum PropertyFilter PropertyFilter;

enum KeyCollectionMode {
    KeyCollectionMode_kOwnOnly,
    KeyCollectionMode_kIncludePrototypes
};
typedef enum KeyCollectionMode KeyCollectionMode;

enum IndexFilter {
    IndexFilter_kIncludeIndices,
    IndexFilter_kSkipIndices
};
typedef enum IndexFilter IndexFilter;

enum IntegrityLevel {
    IntegrityLevel_kFrozen,
    IntegrityLevel_kSealed
};
typedef enum IntegrityLevel IntegrityLevel;

enum PropertyAttribute {
    PropertyAttribute_Absent = -1, /* Instead of Maybe<PropertyAttribute> */
    PropertyAttribute_None = 0,
    PropertyAttribute_ReadOnly = 1 << 0,
    PropertyAttribute_DontEnum = 1 << 1,
    PropertyAttribute_DontDelete = 1 << 2,
};
typedef enum PropertyAttribute PropertyAttribute;

enum PropertyHandlerFlags {
    PropertyHandlerFlags_kNone = 0,
    PropertyHandlerFlags_kAllCanRead = 1,
    PropertyHandlerFlags_kNonMasking = 1 << 1,
    PropertyHandlerFlags_kOnlyInterceptStrings = 1 << 2,
};
typedef enum PropertyHandlerFlags PropertyHandlerFlags;

enum ConstructorBehavior {
    ConstructorBehavior_kThrow,
    ConstructorBehavior_kAllow
};
typedef enum ConstructorBehavior ConstructorBehavior;

enum PromiseRejectEvent {
    PromiseRejectEvent_kPromiseRejectWithNoHandler = 0,
    PromiseRejectEvent_kPromiseHandlerAddedAfterReject = 1
};
typedef enum PromiseRejectEvent PromiseRejectEvent;

#define V8_INTRINSICS_LIST(F) F(ArrayProto_values, array_values_iterator)

enum Intrinsic {
#define V8_DECL_INTRINSIC(name, iname) Intrinsic_k##name,
    V8_INTRINSICS_LIST(V8_DECL_INTRINSIC)
#undef V8_DECL_INTRINSIC
};
typedef enum Intrinsic Intrinsic;

/* Auto-generated forward declarations for class pointers */
#include "v8-glue-decl-generated.h"

struct PropertyCallbackInfo {
    IsolatePtr GetIsolate;
    ValueRef Data;
    ObjectRef This;
    ObjectRef Holder;
    ValueRef ReturnValue;
    bool ShouldThrowOnError;
};

struct FunctionCallbackInfo {
    int Length;
    ValueRef Args;
    ObjectRef This;
    ObjectRef Holder;
    ValueRef NewTarget;
    bool IsConstructCall;
    ValueRef Data;
    IsolatePtr GetIsolate;
    void (*SetReturnValue)(ValueRef);
};

/* These typedefs are just here to give a nicer hint to the user as to
   which type of return value is expected to be set.  For the `Void`
   variant, the SetReturnValue function should not be called.
*/

typedef struct PropertyCallbackInfo *PropertyCallbackInfoPtr_Void;
typedef struct PropertyCallbackInfo *PropertyCallbackInfoPtr_Value;
typedef struct PropertyCallbackInfo *PropertyCallbackInfoPtr_Boolean;
typedef struct PropertyCallbackInfo *PropertyCallbackInfoPtr_Integer;
typedef struct PropertyCallbackInfo *PropertyCallbackInfoPtr_Array;
typedef struct FunctionCallbackInfo *FunctionCallbackInfoPtr_Value;

typedef void (*AccessorGetterCallback)(
    StringRef property,
    PropertyCallbackInfoPtr_Value info);

typedef void (*AccessorNameGetterCallback)(
    NameRef property,
    PropertyCallbackInfoPtr_Value info);

typedef void (*AccessorSetterCallback)(
    StringRef property,
    ValueRef value,
    PropertyCallbackInfoPtr_Void info);

typedef void (*AccessorNameSetterCallback)(
    NameRef property,
    ValueRef value,
    PropertyCallbackInfoPtr_Void info);

typedef void (*FunctionCallback)(
    FunctionCallbackInfoPtr_Value info);

typedef void (*NamedPropertyGetterCallback)(
    StringRef property,
    PropertyCallbackInfoPtr_Value info);

typedef void (*NamedPropertySetterCallback)(
    StringRef property,
    ValueRef value,
    PropertyCallbackInfoPtr_Value info);

typedef void (*NamedPropertyQueryCallback)(
    StringRef property,
    PropertyCallbackInfoPtr_Integer info);

typedef void (*NamedPropertyDeleterCallback)(
    StringRef property,
    PropertyCallbackInfoPtr_Boolean info);

typedef void (*NamedPropertyEnumeratorCallback)(
    PropertyCallbackInfoPtr_Array info);

typedef void (*GenericNamedPropertyGetterCallback)(
    NameRef property,
    PropertyCallbackInfoPtr_Value info);

typedef void (*GenericNamedPropertySetterCallback)(
    NameRef property,
    ValueRef value,
    PropertyCallbackInfoPtr_Value info);

typedef void (*GenericNamedPropertyQueryCallback)(
    NameRef property,
    PropertyCallbackInfoPtr_Integer info);

typedef void (*GenericNamedPropertyDeleterCallback)(
    NameRef property,
    PropertyCallbackInfoPtr_Boolean info);

typedef void (*GenericNamedPropertyEnumeratorCallback)(
    PropertyCallbackInfoPtr_Array info);

typedef void (*GenericNamedPropertyDefinerCallback)(
    NameRef property,
    PropertyDescriptorPtr desc,
    PropertyCallbackInfoPtr_Array info);

typedef void (*GenericNamedPropertyDescriptorCallback)(
    NameRef property,
    PropertyCallbackInfoPtr_Value info);

typedef void (*IndexedPropertyGetterCallback)(
    uint32_t index,
    PropertyCallbackInfoPtr_Value info);

typedef void (*IndexedPropertySetterCallback)(
    uint32_t index,
    ValueRef value,
    PropertyCallbackInfoPtr_Value info);

typedef void (*IndexedPropertyQueryCallback)(
    uint32_t index,
    PropertyCallbackInfoPtr_Integer info);

typedef void (*IndexedPropertyDeleterCallback)(
    uint32_t index,
    PropertyCallbackInfoPtr_Boolean info);

typedef void (*IndexedPropertyEnumeratorCallback)(
    PropertyCallbackInfoPtr_Array info);

typedef void (*IndexedPropertyDefinerCallback)(
    uint32_t index,
    PropertyDescriptorPtr desc,
    PropertyCallbackInfoPtr_Array info);

typedef void (*IndexedPropertyDescriptorCallback)(
    uint32_t index,
    PropertyCallbackInfoPtr_Value info);

typedef bool (*AccessCheckCallback)(
    ContextRef accessing_context,
    ObjectRef accessed_object,
    ValueRef data);

typedef void (*FatalErrorCallback)(
    const char *location,
    const char *message);

typedef void (*OOMErrorCallback)(
    const char *location,
    bool is_heap_oom);

struct NamedPropertyHandlerConfiguration {
    GenericNamedPropertyGetterCallback getter;
    GenericNamedPropertySetterCallback setter;
    GenericNamedPropertyQueryCallback query;
    GenericNamedPropertyDeleterCallback deleter;
    GenericNamedPropertyEnumeratorCallback enumerator;
    GenericNamedPropertyDefinerCallback definer;
    GenericNamedPropertyDescriptorCallback descriptor;
    ValueRef data;
    PropertyHandlerFlags flags;
};
typedef struct NamedPropertyHandlerConfiguration NamedPropertyHandlerConfiguration;

struct IndexedPropertyHandlerConfiguration {
  IndexedPropertyGetterCallback getter;
  IndexedPropertySetterCallback setter;
  IndexedPropertyQueryCallback query;
  IndexedPropertyDeleterCallback deleter;
  IndexedPropertyEnumeratorCallback enumerator;
  IndexedPropertyDefinerCallback definer;
  IndexedPropertyDescriptorCallback descriptor;
  ValueRef data;
  PropertyHandlerFlags flags;
};
typedef struct IndexedPropertyHandlerConfiguration IndexedPropertyHandlerConfiguration;


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
ScriptRef v8_Script_Compile_Origin(RustContext c, ContextRef context, StringRef source, ValueRef resource_name, IntegerRef resource_line_offset, IntegerRef resource_column_offset, BooleanRef resource_is_shared_cross_origin, IntegerRef script_id, BooleanRef resource_is_embedder_debug_script, ValueRef source_map_url, BooleanRef resource_is_opaque);

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
