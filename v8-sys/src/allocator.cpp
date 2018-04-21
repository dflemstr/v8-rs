#include "wrapper.hpp"

namespace rust_v8_impls {

class ArrayBufferAllocator : public v8::ArrayBuffer::Allocator {
public:
  ArrayBufferAllocator(ArrayBufferAllocatorFunctions functions, void *self,
                       v8::ArrayBuffer::Allocator *fallback)
      : _functions(functions), _self(self), _fallback(fallback) {}

  virtual ~ArrayBufferAllocator() {
    if (this->_functions.Destroy) {
      auto super = [this]() { delete this->_fallback; };
      auto thunk = [](void *arg) { (*static_cast<decltype(super) *>(arg))(); };
      this->_functions.Destroy(this->_self, thunk, &super);
    } else {
      delete this->_fallback;
    }
  }

  virtual void *Allocate(size_t length) {
    if (this->_functions.Allocate) {
      auto super = [this](size_t length) {
        return this->_fallback->Allocate(length);
      };
      auto thunk = [](void *arg, size_t length) {
        return (*static_cast<decltype(super) *>(arg))(length);
      };
      return this->_functions.Allocate(this->_self, thunk, &super, length);
    } else {
      return this->_fallback->Allocate(length);
    }
  }

  virtual void *AllocateUninitialized(size_t length) {
    if (this->_functions.AllocateUninitialized) {
      auto super = [this](size_t length) {
        return this->_fallback->AllocateUninitialized(length);
      };
      auto thunk = [](void *arg, size_t length) {
        return (*static_cast<decltype(super) *>(arg))(length);
      };
      return this->_functions.AllocateUninitialized(this->_self, thunk, &super,
                                                    length);
    } else {
      return this->_fallback->AllocateUninitialized(length);
    }
  }

  virtual void *Reserve(size_t length) {
    if (this->_functions.Reserve) {
      auto super = [this](size_t length) {
        return this->_fallback->Reserve(length);
      };
      auto thunk = [](void *arg, size_t length) {
        return (*static_cast<decltype(super) *>(arg))(length);
      };
      return this->_functions.Reserve(this->_self, thunk, &super, length);
    } else {
      return this->_fallback->Reserve(length);
    }
  }

  virtual void Free(void *data, size_t length) {
    if (this->_functions.Free) {
      auto super = [this](void *data, size_t length) {
        return this->_fallback->Free(data, length);
      };
      auto thunk = [](void *arg, void *data, size_t length) {
        (*static_cast<decltype(super) *>(arg))(data, length);
      };
      this->_functions.Free(this->_self, thunk, &super, data, length);
    } else {
      this->_fallback->Free(data, length);
    }
  }

  virtual void Free(void *data, size_t length,
                    v8::ArrayBuffer::Allocator::AllocationMode mode) {
    if (this->_functions.FreeMode) {
      auto super = [this](void *data, size_t length,
                          v8::ArrayBuffer::Allocator::AllocationMode mode) {
        return this->_fallback->Free(data, length, mode);
      };
      auto thunk = [](void *arg, void *data, size_t length,
                      v8::ArrayBuffer::Allocator::AllocationMode mode) {
        (*static_cast<decltype(super) *>(arg))(data, length, mode);
      };
      this->_functions.FreeMode(this->_self, thunk, &super, data, length, mode);
    } else {
      this->_fallback->Free(data, length, mode);
    }
  }

  virtual void
  SetProtection(void *data, size_t length,
                v8::ArrayBuffer::Allocator::Protection protection) {
    if (this->_functions.SetProtection) {
      auto super = [this](void *data, size_t length,
                          v8::ArrayBuffer::Allocator::Protection protection) {
        return this->_fallback->SetProtection(data, length, protection);
      };
      auto thunk = [](void *arg, void *data, size_t length,
                      v8::ArrayBuffer::Allocator::Protection protection) {
        (*static_cast<decltype(super) *>(arg))(data, length, protection);
      };
      this->_functions.SetProtection(this->_self, thunk, &super, data, length,
                                     protection);
    } else {
      this->_fallback->SetProtection(data, length, protection);
    }
  }

private:
  ArrayBufferAllocatorFunctions _functions;
  void *_self;
  v8::ArrayBuffer::Allocator *_fallback;
};

v8::ArrayBuffer::Allocator *
CreateArrayBufferAllocator(ArrayBufferAllocatorFunctions functions,
                           void *data) {
  return new ArrayBufferAllocator(
      functions, data, v8::ArrayBuffer::Allocator::NewDefaultAllocator());
}

void DestroyArrayBufferAllocator(v8::ArrayBuffer::Allocator *allocator) {
  delete allocator;
}

} // namespace rust_v8_impls
