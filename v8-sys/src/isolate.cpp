#include "wrapper.hpp"

void Isolate_SetData(v8::Isolate *self, uint32_t slot, void *data) {
  self->SetData(slot, data);
}

void *Isolate_GetData(v8::Isolate *self, uint32_t slot) {
  return self->GetData(slot);
}

uint32_t Isolate_GetNumberOfDataSlots() {
  return v8::Isolate::GetNumberOfDataSlots();
}
