#include "v8-glue.h"

#include <cstdlib>
#include <cstring>

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

#define UNWRAP_MAYBE_PRIM(PRIM, NAME, MAYBE)    \
    MAYBE unwrap_maybe_##NAME(                  \
        v8::Isolate *isolate,                   \
        v8::Maybe<PRIM> maybe_value)            \
    {                                           \
        PRIM value;                             \
        bool is_set = maybe_value.To(&value);   \
                                                \
        return MAYBE {                          \
            .is_set = is_set,                   \
            .value = value,                     \
        };                                      \
    }

UNWRAP_MAYBE_PRIM(bool, bool, MaybeBool)
UNWRAP_MAYBE_PRIM(int, int, MaybeInt)
UNWRAP_MAYBE_PRIM(unsigned int, uint, MaybeUInt)
UNWRAP_MAYBE_PRIM(long, long, MaybeLong)
UNWRAP_MAYBE_PRIM(unsigned long, ulong, MaybeULong)
UNWRAP_MAYBE_PRIM(uint32_t, u32, MaybeU32)
UNWRAP_MAYBE_PRIM(int32_t, i32, MaybeI32)
UNWRAP_MAYBE_PRIM(uint64_t, u64, MaybeU64)
UNWRAP_MAYBE_PRIM(int64_t, i64, MaybeI64)
UNWRAP_MAYBE_PRIM(double, f64, MaybeF64)

template<typename A> A unwrap(v8::Isolate *isolate, A value) {
    return value;
}

template<typename A> v8::Local<A> wrap(v8::Isolate *isolate,
                                       v8::Persistent<A> *value)
{
    return value->Get(isolate);
}

template<typename A> A wrap(v8::Isolate *isolate, A &&value) {
    return value;
}

void handle_exception(RustContext &c, v8::TryCatch &try_catch) {
    if (try_catch.HasCaught()) {
        *c.exception = unwrap(c.isolate, try_catch.Exception());
        *c.message = unwrap(c.isolate, try_catch.Message());
    }
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

    virtual void CallOnBackgroundThread(v8::Task *task,
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

    virtual void CallOnForegroundThread(v8::Isolate *isolate, v8::Task *task) {
        this->_platform_functions.CallOnForegroundThread(isolate, task);
    }

    virtual void CallDelayedOnForegroundThread(v8::Isolate *isolate, v8::Task *task,
                                               double delay_in_seconds) {
        this->_platform_functions.CallDelayedOnForegroundThread(isolate, task, delay_in_seconds);
    }

    virtual void CallIdleOnForegroundThread(v8::Isolate *isolate, v8::IdleTask *task) {
        this->_platform_functions.CallIdleOnForegroundThread(isolate, task);
    }

    virtual bool IdleTasksEnabled(v8::Isolate *isolate) {
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

PlatformPtr v8_Platform_Create(struct v8_PlatformFunctions platform_functions) {
    return new GluePlatform(platform_functions);
}

void v8_Platform_Destroy(PlatformPtr platform) {
    delete platform;
}

void v8_V8_InitializePlatform(PlatformPtr platform) {
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


ArrayBuffer_AllocatorPtr v8_ArrayBuffer_Allocator_Create(struct v8_AllocatorFunctions allocator_functions) {
    return new GlueAllocator(allocator_functions);

}
void v8_ArrayBuffer_Allocator_Destroy(ArrayBuffer_AllocatorPtr allocator) {
    delete allocator;
}

IsolatePtr v8_Isolate_New(ArrayBuffer_AllocatorPtr allocator) {
    auto params = v8::Isolate::CreateParams();
    params.array_buffer_allocator = allocator;
    return v8::Isolate::New(params);
}

void v8_Isolate_Dispose(IsolatePtr isolate) {
    isolate->Dispose();
}

void v8_Task_Run(TaskPtr task) {
    task->Run();
}

void v8_IdleTask_Run(IdleTaskPtr task, double deadline_in_seconds) {
    task->Run(deadline_in_seconds);
}

#include "v8-glue-generated.cc"

ContextRef v8_Context_New(RustContext c) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    auto result = v8::Context::New(c.isolate);
    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}

StringRef v8_String_NewFromUtf8_Normal(RustContext c, const char *data, int length) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    auto result = v8::String::NewFromUtf8(c.isolate, data, v8::NewStringType::kNormal, length);
    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}

StringRef v8_String_NewFromUtf8_Internalized(RustContext c, const char *data, int length) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    auto result = v8::String::NewFromUtf8(c.isolate, data, v8::NewStringType::kInternalized, length);
    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}

int v8_String_WriteUtf8(RustContext c, StringRef string, char *buffer, int length) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    auto result = wrap(c.isolate, string)->WriteUtf8(buffer, length);
    handle_exception(c, try_catch);
    return result;
}

ScriptRef v8_Script_Compile(RustContext c, ContextRef context, StringRef source) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    v8::Context::Scope context_scope(wrap(c.isolate, context));
    auto result = v8::Script::Compile(wrap(c.isolate, context), wrap(c.isolate, source));
    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}

ValueRef v8_Object_CallAsFunction(RustContext c, ObjectRef self, ContextRef context, ValueRef recv, int argc, ValueRef argv[]) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    v8::Context::Scope context_scope(wrap(c.isolate, context));
    v8::Local<v8::Value> argv_wrapped[argc];
    v8::Local<v8::Value> recv_wrapped;

    for (int i = 0; i < argc; i++) {
        argv_wrapped[i] = wrap(c.isolate, argv[i]);
    }

    if (recv == nullptr) {
        recv_wrapped = v8::Undefined(c.isolate);
    } else {
        recv_wrapped = wrap(c.isolate, recv);
    }

    auto result = wrap(c.isolate, self)->CallAsFunction(wrap(c.isolate, context), recv_wrapped, argc, argv_wrapped);
    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}

ValueRef v8_Object_CallAsConstructor(RustContext c, ObjectRef self, ContextRef context, int argc, ValueRef argv[]) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    v8::Context::Scope context_scope(wrap(c.isolate, context));
    v8::Local<v8::Value> argv_wrapped[argc];

    for (int i = 0; i < argc; i++) {
        argv_wrapped[i] = wrap(c.isolate, argv[i]);
    }

    auto result = wrap(c.isolate, self)->CallAsConstructor(wrap(c.isolate, context), argc, argv_wrapped);
    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}
