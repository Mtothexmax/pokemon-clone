// Reflected components live in headers: the build's codegen only scans headers, which is what makes them editable in the F1 editor and savable in the scene.
#pragma once

#include "ecs/ecs.h"
#include "lib/reflect.h"
#include "lib/typedefs.h"

// tag on the entity the player controls
HZ_REFLECT()
ECS_COMPONENT(Player) {
  b32 sprint_enabled;
};

// scene marker: this entity's transform scale is the size of a solid box the character walks on and collides with
HZ_REFLECT()
ECS_COMPONENT(SolidBox) {
  u8 _;
};
