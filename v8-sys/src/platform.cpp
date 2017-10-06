#include "wrapper.hpp"

namespace rust_v8_impls {

class Platform : public v8::Platform {
public:
  Platform(PlatformFunctions functions, void *data)
      : _functions(functions), _data(data) {}

  virtual ~Platform() { this->_functions.Destroy(this->_data); }

  virtual size_t NumberOfAvailableBackgroundThreads() {
    return this->_functions.NumberOfAvailableBackgroundThreads(this->_data);
  }

  virtual void
  CallOnBackgroundThread(v8::Task *task,
                         v8::Platform::ExpectedRuntime expected_runtime) {
    this->_functions.CallOnBackgroundThread(this->_data, task,
                                            expected_runtime);
  }

  virtual void CallOnForegroundThread(v8::Isolate *isolate, v8::Task *task) {
    this->_functions.CallOnForegroundThread(this->_data, isolate, task);
  }

  virtual void CallDelayedOnForegroundThread(v8::Isolate *isolate,
                                             v8::Task *task,
                                             double delay_in_seconds) {
    this->_functions.CallDelayedOnForegroundThread(this->_data, isolate, task,
                                                   delay_in_seconds);
  }

  virtual void CallIdleOnForegroundThread(v8::Isolate *isolate,
                                          v8::IdleTask *task) {
    this->_functions.CallIdleOnForegroundThread(this->_data, isolate, task);
  }

  virtual bool IdleTasksEnabled(v8::Isolate *isolate) {
    return this->_functions.IdleTasksEnabled(this->_data, isolate);
  }

  virtual double MonotonicallyIncreasingTime() {
    return this->_functions.MonotonicallyIncreasingTime(this->_data);
  }

  virtual double CurrentClockTimeMillis() {
    return 0.0;
  }

  virtual v8::TracingController *GetTracingController() {
    return nullptr;
  }

private:
  PlatformFunctions _functions;
  void *_data;
};

v8::Platform *CreatePlatform(PlatformFunctions functions, void *data) {
  return new Platform(functions, data);
}

void DestroyPlatform(v8::Platform *platform) { delete platform; }

void DestroyTask(v8::Task *task) { delete task; }

void DestroyIdleTask(v8::IdleTask *idle_task) { delete idle_task; }

} // namespace rust_v8_impls
