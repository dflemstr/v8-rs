#include <v8-platform.h>
#include <v8.h>

namespace rust_v8_impls {

struct ArrayBufferAllocatorFunctions {
  void (*Destroy)(void *, void (*)(void *), void *);
  void *(*Allocate)(void *, void *(*)(void *, size_t), void *, size_t);
  void *(*AllocateUninitialized)(void *, void *(*)(void *, size_t), void *,
                                 size_t);
  void *(*Reserve)(void *, void *(*)(void *, size_t), void *, size_t);
  void (*Free)(void *, void (*)(void *, void *, size_t), void *, void *,
               size_t);
  void (*FreeMode)(void *,
                   void (*)(void *, void *, size_t,
                            v8::ArrayBuffer::Allocator::AllocationMode),
                   void *, void *, size_t,
                   v8::ArrayBuffer::Allocator::AllocationMode);
  void (*SetProtection)(void *,
                        void (*)(void *, void *, size_t,
                                 v8::ArrayBuffer::Allocator::Protection),
                        void *, void *, size_t,
                        v8::ArrayBuffer::Allocator::Protection);
};

struct PlatformFunctions {
  void (*Destroy)(void *);
  size_t (*NumberOfAvailableBackgroundThreads)(void *);
  void (*CallOnBackgroundThread)(void *, v8::Task *,
                                 v8::Platform::ExpectedRuntime);
  void (*CallOnForegroundThread)(void *, v8::Isolate *, v8::Task *);
  void (*CallDelayedOnForegroundThread)(void *, v8::Isolate *, v8::Task *,
                                        double);
  void (*CallIdleOnForegroundThread)(void *, v8::Isolate *, v8::IdleTask *);
  bool (*IdleTasksEnabled)(void *, v8::Isolate *);
  double (*MonotonicallyIncreasingTime)(void *);
};

v8::ArrayBuffer::Allocator *
CreateArrayBufferAllocator(ArrayBufferAllocatorFunctions functions, void *data);

v8::Platform *CreatePlatform(PlatformFunctions functions, void *data);

/**
 * <div rustbindgen replaces="v8::JitCodeEvent"></div>
 */
typedef void JitCodeEvent;

} // namespace rust_v8_impls
