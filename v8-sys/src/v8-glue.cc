#include "v8-glue.h"

#include <cstdlib>
#include <cstring>


template<typename A> v8::Persistent<A> *unwrap(v8::Isolate *isolate,
                                               v8::Local<A> value)
{
    if (value.IsEmpty()) {
        return nullptr;
    } else {
        return new v8::Persistent<A>(isolate, value);
    }
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

PropertyAttribute unwrap(
    v8::Isolate *isolate,
    v8::Maybe<v8::PropertyAttribute> maybe_value) {
    v8::PropertyAttribute value;

    if (maybe_value.To(&value)) {
        PropertyAttribute result = PropertyAttribute_None;

        if (value & v8::PropertyAttribute::ReadOnly) {
            result = PropertyAttribute(result | PropertyAttribute_ReadOnly);
        }

        if (value & v8::PropertyAttribute::DontEnum) {
            result = PropertyAttribute(result | PropertyAttribute_DontEnum);
        }

        if (value & v8::PropertyAttribute::DontDelete) {
            result = PropertyAttribute(result | PropertyAttribute_DontDelete);
        }

        return result;
    } else {
        return PropertyAttribute_Absent;
    }
}

PromiseRejectEvent unwrap(
    v8::Isolate *isolate,
    v8::PromiseRejectEvent value) {
    switch (value) {
    default:
    case v8::PromiseRejectEvent::kPromiseRejectWithNoHandler:
        return PromiseRejectEvent_kPromiseRejectWithNoHandler;
    case v8::kPromiseHandlerAddedAfterReject:
        return PromiseRejectEvent_kPromiseHandlerAddedAfterReject;
    }
}

template<typename A> A unwrap(v8::Isolate *isolate, A &&value) {
    return value;
}

template<typename A> v8::Local<A> wrap(v8::Isolate *isolate,
                                       v8::Persistent<A> *value)
{
    if (value) {
        return value->Get(isolate);
    } else {
        return v8::Local<A>();
    }
}

v8::AccessControl wrap(v8::Isolate *isolate, AccessControl value) {
    v8::AccessControl result = v8::AccessControl::DEFAULT;

    if (value & AccessControl_ALL_CAN_READ) {
        result = v8::AccessControl(result | v8::AccessControl::ALL_CAN_READ);
    }

    if (value & AccessControl_ALL_CAN_WRITE) {
        result = v8::AccessControl(result | v8::AccessControl::ALL_CAN_WRITE);
    }

    if (value & AccessControl_PROHIBITS_OVERWRITING) {
        result = v8::AccessControl(result | v8::AccessControl::PROHIBITS_OVERWRITING);
    }

    return result;
}

v8::PropertyFilter wrap(v8::Isolate *isolate, PropertyFilter value) {
    v8::PropertyFilter result = v8::PropertyFilter::ALL_PROPERTIES;

    if (value & PropertyFilter_ONLY_WRITABLE) {
        result = v8::PropertyFilter(result | v8::PropertyFilter::ONLY_WRITABLE);
    }

    if (value & PropertyFilter_ONLY_ENUMERABLE) {
        result = v8::PropertyFilter(result | v8::PropertyFilter::ONLY_ENUMERABLE);
    }

    if (value & PropertyFilter_ONLY_CONFIGURABLE) {
        result = v8::PropertyFilter(result | v8::PropertyFilter::ONLY_CONFIGURABLE);
    }

    if (value & PropertyFilter_SKIP_STRINGS) {
        result = v8::PropertyFilter(result | v8::PropertyFilter::SKIP_STRINGS);
    }

    if (value & PropertyFilter_SKIP_SYMBOLS) {
        result = v8::PropertyFilter(result | v8::PropertyFilter::SKIP_SYMBOLS);
    }

    return result;
}

v8::KeyCollectionMode wrap(v8::Isolate *isolate, KeyCollectionMode value) {
    switch (value) {
    default:
    case KeyCollectionMode_kOwnOnly:
        return v8::KeyCollectionMode::kOwnOnly;
    case KeyCollectionMode_kIncludePrototypes:
        return v8::KeyCollectionMode::kIncludePrototypes;
    }
}

v8::IndexFilter wrap(v8::Isolate *isolate, IndexFilter value) {
    switch (value) {
    default:
    case IndexFilter_kIncludeIndices:
        return v8::IndexFilter::kIncludeIndices;
    case IndexFilter_kSkipIndices:
        return v8::IndexFilter::kSkipIndices;
    }
}

v8::IntegrityLevel wrap(v8::Isolate *isolate, IntegrityLevel value) {
    switch (value) {
    default:
    case IntegrityLevel_kFrozen:
        return v8::IntegrityLevel::kFrozen;
    case IntegrityLevel_kSealed:
        return v8::IntegrityLevel::kSealed;
    }
}

v8::PropertyAttribute wrap(v8::Isolate *isolate, PropertyAttribute value) {
    if (value == PropertyAttribute_Absent) {
        return v8::PropertyAttribute::None;
    }

    v8::PropertyAttribute result = v8::PropertyAttribute::None;

    if (value & PropertyAttribute_ReadOnly) {
        result = v8::PropertyAttribute(result | v8::PropertyAttribute::ReadOnly);
    }

    if (value & PropertyAttribute_DontEnum) {
        result = v8::PropertyAttribute(result | v8::PropertyAttribute::DontEnum);
    }

    if (value & PropertyAttribute_DontDelete) {
        result = v8::PropertyAttribute(result | v8::PropertyAttribute::DontDelete);
    }

    return result;
}

v8::PropertyHandlerFlags wrap(v8::Isolate *isolate, PropertyHandlerFlags value) {
    v8::PropertyHandlerFlags result = v8::PropertyHandlerFlags::kNone;

    if (value & PropertyHandlerFlags_kAllCanRead) {
        result = v8::PropertyHandlerFlags(int(result) | int(v8::PropertyHandlerFlags::kAllCanRead));
    }

    if (value & PropertyHandlerFlags_kNonMasking) {
        result = v8::PropertyHandlerFlags(int(result) | int(v8::PropertyHandlerFlags::kNonMasking));
    }

    if (value & PropertyHandlerFlags_kOnlyInterceptStrings) {
        result = v8::PropertyHandlerFlags(int(result) | int(v8::PropertyHandlerFlags::kOnlyInterceptStrings));
    }

    return result;
}

v8::ConstructorBehavior wrap(v8::Isolate *isolate, ConstructorBehavior value) {
    switch (value) {
    default:
    case ConstructorBehavior_kThrow:
        return v8::ConstructorBehavior::kThrow;
    case ConstructorBehavior_kAllow:
        return v8::ConstructorBehavior::kAllow;
    }
}

v8::PromiseRejectEvent wrap(v8::Isolate *isolate, PromiseRejectEvent value) {
    switch (value) {
    default:
    case PromiseRejectEvent_kPromiseRejectWithNoHandler:
        return v8::PromiseRejectEvent::kPromiseRejectWithNoHandler;
    case PromiseRejectEvent_kPromiseHandlerAddedAfterReject:
        return v8::PromiseRejectEvent::kPromiseHandlerAddedAfterReject;
    }
}

v8::Intrinsic wrap(v8::Isolate *isolate, Intrinsic value) {
    switch (value) {
    default:
#define V8_SWITCH_INTRINSIC(name, iname) case Intrinsic_k##name: return v8::Intrinsic::k##name;
        V8_INTRINSICS_LIST(V8_SWITCH_INTRINSIC)
#undef V8_SWITCH_INTRINSIC
    }
}

v8::ArrayBufferCreationMode wrap(v8::Isolate *isolate, ArrayBufferCreationMode value) {
    switch (value) {
    default:
    case ArrayBufferCreationMode_kInternalized:
        return v8::ArrayBufferCreationMode::kInternalized;
    case ArrayBufferCreationMode_kExternalized:
        return v8::ArrayBufferCreationMode::kExternalized;
    }
}

template<typename A>
PropertyCallbackInfo build_callback_info(
    const v8::PropertyCallbackInfo<A> &info,
    v8::Local<v8::Value> data) {

    v8::Isolate *isolate = info.GetIsolate();

    PropertyCallbackInfo result = PropertyCallbackInfo {
        .GetIsolate = isolate,
        .Data = unwrap(isolate, data),
        .This = unwrap(isolate, info.This()),
        .Holder = unwrap(isolate, info.Holder()),
        .ReturnValue = nullptr,
        .ShouldThrowOnError = info.ShouldThrowOnError(),
    };

    return result;
}

template<typename A>
FunctionCallbackInfo build_callback_info(
    const v8::FunctionCallbackInfo<A> &info,
    v8::Local<v8::Value> data) {

    v8::Isolate *isolate = info.GetIsolate();

    int length = info.Length();
    ValueRef *args = new ValueRef[length];

    for (int i = 0; i < length; i++) {
        v8::Local<v8::Value> arg = info[i];
        ValueRef unwrapped_arg = unwrap(isolate, arg);
        args[i] = unwrapped_arg;
    }

    FunctionCallbackInfo result = FunctionCallbackInfo {
        .Length = length,
        .Args = args,
        .This = unwrap(isolate, info.This()),
        .Holder = unwrap(isolate, info.Holder()),
        .NewTarget = unwrap(isolate, info.NewTarget()),
        .IsConstructCall = info.IsConstructCall(),
        .Data = unwrap(isolate, data),
        .GetIsolate = isolate,
        .ReturnValue = nullptr,
    };

    return result;
}

enum class PropertyHandlerFields {
    Getter, Setter, Query, Deleter, Enumerator, Data, Flags, Max
};

void generic_named_property_handler_getter(
    v8::Local<v8::Name> property,
    const v8::PropertyCallbackInfo<v8::Value> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    GenericNamedPropertyGetterCallback getter =
        (GenericNamedPropertyGetterCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Getter);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    getter(unwrap(isolate, property), &callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(wrap(isolate, callback_info.ReturnValue));
    }
}

void generic_named_property_handler_setter(
    v8::Local<v8::Name> property,
    v8::Local<v8::Value> value,
    const v8::PropertyCallbackInfo<v8::Value> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    GenericNamedPropertySetterCallback setter =
        (GenericNamedPropertySetterCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Setter);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    setter(unwrap(isolate, property), unwrap(isolate, value), &callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(wrap(isolate, callback_info.ReturnValue));
    }
}

void generic_named_property_handler_query(
    v8::Local<v8::Name> property,
    const v8::PropertyCallbackInfo<v8::Integer> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    GenericNamedPropertyQueryCallback query =
        (GenericNamedPropertyQueryCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Query);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    query(unwrap(isolate, property), &callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(v8::Local<v8::Integer>::Cast(wrap(isolate, callback_info.ReturnValue)));
    }
}

void generic_named_property_handler_deleter(
    v8::Local<v8::Name> property,
    const v8::PropertyCallbackInfo<v8::Boolean> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    GenericNamedPropertyDeleterCallback deleter =
        (GenericNamedPropertyDeleterCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Deleter);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    deleter(unwrap(isolate, property), &callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(v8::Local<v8::Boolean>::Cast(wrap(isolate, callback_info.ReturnValue)));
    }
}

void generic_named_property_handler_enumerator(
    const v8::PropertyCallbackInfo<v8::Array> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    GenericNamedPropertyEnumeratorCallback enumerator =
        (GenericNamedPropertyEnumeratorCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Enumerator);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    enumerator(&callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(v8::Local<v8::Array>::Cast(wrap(isolate, callback_info.ReturnValue)));
    }
}

v8::NamedPropertyHandlerConfiguration wrap(
    v8::Isolate *isolate,
    NamedPropertyHandlerConfiguration value) {
    v8::Local<v8::ObjectTemplate> outer_data_template =
        v8::ObjectTemplate::New(isolate);
    outer_data_template->SetInternalFieldCount((int) PropertyHandlerFields::Max);
    v8::Local<v8::Object> outer_data = outer_data_template->NewInstance();

    v8::GenericNamedPropertyGetterCallback getter;
    v8::GenericNamedPropertySetterCallback setter;
    v8::GenericNamedPropertyQueryCallback query;
    v8::GenericNamedPropertyDeleterCallback deleter;
    v8::GenericNamedPropertyEnumeratorCallback enumerator;

    if (value.getter) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Getter, (void *) value.getter);
        getter = generic_named_property_handler_getter;
    } else {
        getter = nullptr;
    }

    if (value.setter) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Setter, (void *) value.setter);
        setter = generic_named_property_handler_setter;
    } else {
        setter = nullptr;
    }

    if (value.query) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Query, (void *) value.query);
        query = generic_named_property_handler_query;
    } else {
        query = nullptr;
    }

    if (value.deleter) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Deleter, (void *) value.deleter);
        deleter = generic_named_property_handler_deleter;
    } else {
        deleter = nullptr;
    }

    if (value.enumerator) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Enumerator, (void *) value.enumerator);
        enumerator = generic_named_property_handler_enumerator;
    } else {
        enumerator = nullptr;
    }

    outer_data->SetInternalField((int) PropertyHandlerFields::Data, wrap(isolate, value.data));

    return v8::NamedPropertyHandlerConfiguration(
        getter,
        setter,
        query,
        deleter,
        enumerator,
        outer_data,
        wrap(isolate, value.flags));
}

void indexed_property_handler_getter(
    uint32_t index,
    const v8::PropertyCallbackInfo<v8::Value> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    IndexedPropertyGetterCallback getter =
        (IndexedPropertyGetterCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Getter);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    getter(index, &callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(wrap(isolate, callback_info.ReturnValue));
    }
}

void indexed_property_handler_setter(
    uint32_t index,
    v8::Local<v8::Value> value,
    const v8::PropertyCallbackInfo<v8::Value> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    IndexedPropertySetterCallback setter =
        (IndexedPropertySetterCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Setter);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    setter(index, unwrap(isolate, value), &callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(wrap(isolate, callback_info.ReturnValue));
    }
}

void indexed_property_handler_query(
    uint32_t index,
    const v8::PropertyCallbackInfo<v8::Integer> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    IndexedPropertyQueryCallback query =
        (IndexedPropertyQueryCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Query);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    query(index, &callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(v8::Local<v8::Integer>::Cast(wrap(isolate, callback_info.ReturnValue)));
    }
}

void indexed_property_handler_deleter(
    uint32_t index,
    const v8::PropertyCallbackInfo<v8::Boolean> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    IndexedPropertyDeleterCallback deleter =
        (IndexedPropertyDeleterCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Deleter);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    deleter(index, &callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(v8::Local<v8::Boolean>::Cast(wrap(isolate, callback_info.ReturnValue)));
    }
}

void indexed_property_handler_enumerator(
    const v8::PropertyCallbackInfo<v8::Array> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());
    IndexedPropertyEnumeratorCallback enumerator =
        (IndexedPropertyEnumeratorCallback)
        outer_data->GetAlignedPointerFromInternalField((int) PropertyHandlerFields::Enumerator);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) PropertyHandlerFields::Data);
    PropertyCallbackInfo callback_info = build_callback_info(info, data);

    enumerator(&callback_info);

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(v8::Local<v8::Array>::Cast(wrap(isolate, callback_info.ReturnValue)));
    }
}

v8::IndexedPropertyHandlerConfiguration wrap(
    v8::Isolate *isolate,
    IndexedPropertyHandlerConfiguration value) {
    v8::Local<v8::ObjectTemplate> outer_data_template =
        v8::ObjectTemplate::New(isolate);
    outer_data_template->SetInternalFieldCount((int) PropertyHandlerFields::Max);
    v8::Local<v8::Object> outer_data = outer_data_template->NewInstance();

    v8::IndexedPropertyGetterCallback getter;
    v8::IndexedPropertySetterCallback setter;
    v8::IndexedPropertyQueryCallback query;
    v8::IndexedPropertyDeleterCallback deleter;
    v8::IndexedPropertyEnumeratorCallback enumerator;

    if (value.getter) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Getter, (void *) value.getter);
        getter = indexed_property_handler_getter;
    } else {
        getter = nullptr;
    }

    if (value.setter) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Setter, (void *) value.setter);
        setter = indexed_property_handler_setter;
    } else {
        setter = nullptr;
    }

    if (value.query) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Query, (void *) value.query);
        query = indexed_property_handler_query;
    } else {
        query = nullptr;
    }

    if (value.deleter) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Deleter, (void *) value.deleter);
        deleter = indexed_property_handler_deleter;
    } else {
        deleter = nullptr;
    }

    if (value.enumerator) {
        outer_data->SetAlignedPointerInInternalField((int) PropertyHandlerFields::Enumerator, (void *) value.enumerator);
        enumerator = indexed_property_handler_enumerator;
    } else {
        enumerator = nullptr;
    }

    outer_data->SetInternalField((int) PropertyHandlerFields::Data, wrap(isolate, value.data));

    return v8::IndexedPropertyHandlerConfiguration(
        getter,
        setter,
        query,
        deleter,
        enumerator,
        outer_data,
        wrap(isolate, value.flags));
}

template<typename A> A wrap(v8::Isolate *isolate, A value) {
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
        default:
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

void v8_Isolate_SetCaptureStackTraceForUncaughtExceptions_Overview(IsolatePtr self, bool capture, int frame_limit) {
    self->SetCaptureStackTraceForUncaughtExceptions(capture, frame_limit, v8::StackTrace::kOverview);
}

void v8_Isolate_SetCaptureStackTraceForUncaughtExceptions_Detailed(IsolatePtr self, bool capture, int frame_limit) {
    self->SetCaptureStackTraceForUncaughtExceptions(capture, frame_limit, v8::StackTrace::kDetailed);
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


ScriptRef v8_Script_Compile_Origin(
    RustContext c,
    ContextRef context,
    StringRef source,
    ValueRef resource_name,
    IntegerRef resource_line_offset,
    IntegerRef resource_column_offset,
    BooleanRef resource_is_shared_cross_origin,
    IntegerRef script_id,
    BooleanRef resource_is_embedder_debug_script,
    ValueRef source_map_url,
    BooleanRef resource_is_opaque) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    v8::Context::Scope context_scope(wrap(c.isolate, context));

    v8::ScriptOrigin origin(
        wrap(c.isolate, resource_name),
        wrap(c.isolate, resource_line_offset),
        wrap(c.isolate, resource_column_offset),
        wrap(c.isolate, resource_is_shared_cross_origin),
        wrap(c.isolate, script_id),
        wrap(c.isolate, resource_is_embedder_debug_script),
        wrap(c.isolate, source_map_url),
        wrap(c.isolate, resource_is_opaque));

    auto result = v8::Script::Compile(
        wrap(c.isolate, context),
        wrap(c.isolate, source),
        &origin);

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

enum class FunctionHandlerFields {
    Callback, Data, Max
};


void function_callback(const v8::FunctionCallbackInfo<v8::Value> &info) {
    v8::Isolate *isolate = info.GetIsolate();
    v8::HandleScope scope(isolate);
    v8::Local<v8::Object> outer_data =
        v8::Local<v8::Object>::Cast(info.Data());

    FunctionCallback callback =
        (FunctionCallback)
        outer_data->GetAlignedPointerFromInternalField((int) FunctionHandlerFields::Callback);
    v8::Local<v8::Value> data = outer_data->GetInternalField((int) FunctionHandlerFields::Data);
    FunctionCallbackInfo callback_info = build_callback_info(info, data);

    callback(&callback_info);

    delete[] callback_info.Args;

    if (callback_info.ReturnValue) {
        info.GetReturnValue().Set(wrap(isolate, callback_info.ReturnValue));
    }
}

FunctionRef v8_Function_New(
    RustContext c,
    ContextRef context,
    FunctionCallback wrapped_callback,
    ValueRef data,
    int length,
    ConstructorBehavior behavior) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    v8::Local<v8::ObjectTemplate> outer_data_template =
        v8::ObjectTemplate::New(c.isolate);
    outer_data_template->SetInternalFieldCount((int) FunctionHandlerFields::Max);
    v8::Local<v8::Object> outer_data =
        outer_data_template->NewInstance(wrap(c.isolate, context)).ToLocalChecked();

    v8::FunctionCallback callback;

    if (wrapped_callback) {
        outer_data->SetAlignedPointerInInternalField((int) FunctionHandlerFields::Callback, (void *) wrapped_callback);
        callback = function_callback;
    }

    outer_data->SetInternalField((int) FunctionHandlerFields::Data, wrap(c.isolate, data));

    auto result = v8::Function::New(wrap(c.isolate, context), callback, outer_data, length, wrap(c.isolate, behavior));

    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}

ObjectRef v8_Function_NewInstance(
    RustContext c,
    FunctionRef self,
    ContextRef context,
    int argc,
    ValueRef argv[]) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);
    v8::Context::Scope context_scope(wrap(c.isolate, context));

    v8::Local<v8::Value> argv_wrapped[argc];

    for (int i = 0; i < argc; i++) {
        argv_wrapped[i] = wrap(c.isolate, argv[i]);
    }

    auto result = wrap(c.isolate, self)->NewInstance(wrap(c.isolate, context), argc, argv_wrapped);

    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}

ValueRef v8_Function_Call(
    RustContext c,
    FunctionRef self,
    ContextRef context,
    ValueRef recv,
    int argc,
    ValueRef argv[]) {
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

    auto result = wrap(c.isolate, self)->Call(wrap(c.isolate, context), recv_wrapped, argc, argv_wrapped);

    handle_exception(c, try_catch);
    return unwrap(c.isolate, result);
}

void v8_Template_SetNativeDataProperty(
    RustContext c,
    TemplateRef self,
    StringRef name,
    AccessorGetterCallback getter,
    AccessorSetterCallback setter,
    ValueRef data,
    PropertyAttribute attribute,
    AccessorSignatureRef signature,
    AccessControl settings) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);

    handle_exception(c, try_catch);
}

void v8_ObjectTemplate_SetAccessor(
    RustContext c,
    ObjectTemplateRef self,
    StringRef name,
    AccessorGetterCallback getter,
    AccessorSetterCallback setter,
    ValueRef data,
    AccessControl settings,
    PropertyAttribute attribute,
    AccessorSignatureRef signature) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);

    handle_exception(c, try_catch);
}

void v8_ObjectTemplate_SetAccessor_Name(
    RustContext c,
    ObjectTemplateRef self,
    StringRef name,
    AccessorNameGetterCallback getter,
    AccessorNameSetterCallback setter,
    ValueRef data,
    AccessControl settings,
    PropertyAttribute attribute,
    AccessorSignatureRef signature) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);

    handle_exception(c, try_catch);
}

void v8_ObjectTemplate_SetCallAsFunctionHandler(
    RustContext c,
    ObjectTemplateRef self,
    FunctionCallback callback,
    ValueRef data) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);

    handle_exception(c, try_catch);
}

void v8_ObjectTemplate_SetAccessCheckCallback(
    RustContext c,
    ObjectTemplateRef self,
    AccessCheckCallback callback,
    ValueRef data) {
    v8::HandleScope scope(c.isolate);
    v8::TryCatch try_catch(c.isolate);

    handle_exception(c, try_catch);
}
