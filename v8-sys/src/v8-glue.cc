#include "v8-glue.h"

#include <cstdlib>
#include <cstring>

#include "libplatform/libplatform.h"

template<typename A> v8::Persistent<A> *unwrap(v8::Isolate *isolate,
                                               v8::Local<A> value)
{
    return new v8::Persistent<A>(isolate, value);
}

template<typename A> v8::Persistent<A> *unwrap(v8::Isolate *isolate,
                                               v8::MaybeLocal<A> value)
{
    v8::Local<A> local;

    if (value.ToLocal(&local)) {
        return new v8::Persistent<A>(isolate, local);
    } else {
        return nullptr;
    }
}

#define UNWRAP_PRIMITIVE(PRIM, MAYBE) MAYBE unwrap_##PRIM(              \
        v8::Isolate *isolate,                                           \
        v8::Maybe<PRIM> maybe_value)                                    \
    {                                                                   \
        PRIM value;                                                     \
        bool is_set = maybe_value.To(&value);                           \
                                                                        \
        return MAYBE { is_set, value };                                 \
    }

UNWRAP_PRIMITIVE(bool, MaybeBool)
UNWRAP_PRIMITIVE(double, MaybeF64)
UNWRAP_PRIMITIVE(uint32_t, MaybeU32)
UNWRAP_PRIMITIVE(int32_t, MaybeI32)
UNWRAP_PRIMITIVE(uint64_t, MaybeU64)
UNWRAP_PRIMITIVE(int64_t, MaybeI64)
UNWRAP_PRIMITIVE(int, MaybeInt)

template<typename A> A unwrap(v8::Isolate *isolate, A value) {
    return value;
}

template<typename A> v8::Local<A> wrap(
                                       v8::Isolate *isolate,
                                       v8::Persistent<A> *value)
{
    return value->Get(isolate);
}

template<typename A> A wrap(v8::Isolate *isolate, A &&value) {
    return value;
}

class GluePlatform : public v8::Platform {
public:
    GluePlatform(v8_PlatformFunctions platform_functions)
        : _platform_functions(platform_functions)
    {}

    virtual ~GluePlatform() {
        this->_platform_functions.Destroy();
    }

    virtual size_t NumberOfAvailableBackgroundThreads() {
        return this->_platform_functions.NumberOfAvailableBackgroundThreads();
    }

    virtual void CallOnBackgroundThread(Task* task,
                                        v8::Platform::ExpectedRuntime expected_runtime) {
        v8_ExpectedRuntime rt;

        switch (expected_runtime) {
        case v8::Platform::kShortRunningTask:
            rt = SHORT_RUNNING_TASK;
            break;
        case v8::Platform::kLongRunningTask:
            rt = LONG_RUNNING_TASK;
            break;
        }

        this->_platform_functions.CallOnBackgroundThread(task, rt);
    }

    virtual void CallOnForegroundThread(Isolate* isolate, Task* task) {
        this->_platform_functions.CallOnForegroundThread(isolate, task);
    }

    virtual void CallDelayedOnForegroundThread(Isolate* isolate, Task* task,
                                               double delay_in_seconds) {
        this->_platform_functions.CallDelayedOnForegroundThread(isolate, task, delay_in_seconds);
    }

    virtual void CallIdleOnForegroundThread(Isolate* isolate, IdleTask* task) {
        this->_platform_functions.CallIdleOnForegroundThread(isolate, task);
    }

    virtual bool IdleTasksEnabled(Isolate* isolate) {
        return this->_platform_functions.IdleTasksEnabled(isolate);
    }

    virtual double MonotonicallyIncreasingTime() {
        return this->_platform_functions.MonotonicallyIncreasingTime();
    }

private:
    v8_PlatformFunctions _platform_functions;
};

class GlueAllocator : public v8::ArrayBuffer::Allocator {
public:
    GlueAllocator(v8_AllocatorFunctions allocator_functions)
        : _allocator_functions(allocator_functions)
    {}

    virtual void* Allocate(size_t length) {
        return this->_allocator_functions.Allocate(length);
    }

    virtual void* AllocateUninitialized(size_t length) {
        return this->_allocator_functions.AllocateUninitialized(length);
    }

    virtual void Free(void* data, size_t length) {
        this->_allocator_functions.Free(data, length);
    }

private:
    v8_AllocatorFunctions _allocator_functions;
};

Platform *v8_Platform_Create(struct v8_PlatformFunctions platform_functions) {
    return new GluePlatform(platform_functions);
}

void v8_Platform_Destroy(Platform *platform) {
    delete platform;
}

void v8_V8_InitializePlatform(Platform *platform) {
    return v8::V8::InitializePlatform(platform);
}

void v8_V8_InitializeICU() {
    v8::V8::InitializeICU();
}

void v8_V8_Initialize() {
    v8::V8::Initialize();
}

void v8_V8_Dispose() {
    v8::V8::Dispose();
}

void v8_V8_ShutdownPlatform() {
    v8::V8::ShutdownPlatform();
}


ArrayBuffer_Allocator *v8_ArrayBuffer_Allocator_Create(struct v8_AllocatorFunctions allocator_functions) {
    return new GlueAllocator(allocator_functions);

}
void v8_ArrayBuffer_Allocator_Destroy(ArrayBuffer_Allocator *allocator) {
    delete allocator;
}

Isolate *v8_Isolate_New(ArrayBuffer_Allocator *allocator) {
    auto params = v8::Isolate::CreateParams();
    params.array_buffer_allocator = allocator;
    return v8::Isolate::New(params);
}

void v8_Isolate_Dispose(Isolate *isolate) {
    isolate->Dispose();
}

void v8_Task_Run(Task *task) {
    task->Run();
}

void v8_IdleTask_Run(IdleTask *task, double deadline_in_seconds) {
    task->Run(deadline_in_seconds);
}

Context *v8_Context_New(Isolate *isolate) {
    v8::HandleScope scope(isolate);
    return unwrap(isolate, v8::Context::New(isolate));
}

void v8_Context_Enter(Isolate *isolate, Context *context) {
    v8::HandleScope scope(isolate);
    wrap(isolate, context)->Enter();
}

void v8_Context_Exit(Isolate *isolate, Context *context) {
    v8::HandleScope scope(isolate);
    v8::Context::Scope context_scope(wrap(isolate, context));
    wrap(isolate, context)->Exit();
}

String *v8_String_NewFromUtf8_Normal(Isolate *isolate, const char *data, int length) {
    v8::HandleScope scope(isolate);
    return unwrap(isolate, v8::String::NewFromUtf8(isolate, data, v8::NewStringType::kNormal, length));
}

String *v8_String_NewFromUtf8_Internalized(Isolate *isolate, const char *data, int length) {
    v8::HandleScope scope(isolate);
    return unwrap(isolate, v8::String::NewFromUtf8(isolate, data, v8::NewStringType::kInternalized, length));
}

int v8_String_WriteUtf8(Isolate *isolate, String *string, char *buffer, int length) {
    v8::HandleScope scope(isolate);
    return wrap(isolate, string)->WriteUtf8(buffer, length);
}

Script *v8_Script_Compile(Isolate *isolate, Context *context, String *source) {
    v8::HandleScope scope(isolate);
    v8::Context::Scope context_scope(wrap(isolate, context));
    return unwrap(isolate, v8::Script::Compile(wrap(isolate, context), wrap(isolate, source)));
}

Value *v8_Object_CallAsFunction(Isolate *isolate, Object *self, Context *context, Value *recv, int argc, Value *argv[]) {
    v8::HandleScope scope(isolate);
    v8::Context::Scope context_scope(wrap(isolate, context));
    v8::Local<v8::Value> argv_wrapped[argc];

    for (int i = 0; i < argc; i++) {
        argv_wrapped[i] = wrap(isolate, argv[i]);
    }

    return unwrap(isolate, wrap(isolate, self)->CallAsFunction(wrap(isolate, context), wrap(isolate, recv), argc, argv_wrapped));
}

Value *v8_Object_CallAsConstructor(Isolate *isolate, Object *self, Context *context, Value *recv, int argc, Value *argv[]) {
    v8::HandleScope scope(isolate);
    v8::Context::Scope context_scope(wrap(isolate, context));
    v8::Local<v8::Value> argv_wrapped[argc];

    for (int i = 0; i < argc; i++) {
        argv_wrapped[i] = wrap(isolate, argv[i]);
    }

    return unwrap(isolate, wrap(isolate, self)->CallAsConstructor(wrap(isolate, context), argc, argv_wrapped));
}

#include "v8-glue-generated.cc"
