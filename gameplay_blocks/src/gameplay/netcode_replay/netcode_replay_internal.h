// The multiplayer block's hooks into the replay block: module-internal, not for games.
#pragma once

#include "gameplay/multiplayer/multiplayer.h"

typedef struct NetcodeReplay NetcodeReplay;

// multiplayer.c: the Multiplayer's replay state, NULL until netcode_replay_init
NetcodeReplay **multiplayer_replay_slot(Multiplayer *m);
// multiplayer.c: MultiplayerDesc.registry
const NetcodeRegistry *multiplayer_registry(const Multiplayer *m);
// main lane, from multiplayer_destroy before the netcode closes: the replay, the recording and the replay world end
void netcode_replay_destroy(NetcodeReplay *r);
// main lane, from a role switch before the netcode closes: the replay and the recording end; the next netcode's client is recorded afresh
void netcode_replay_reset(NetcodeReplay *r);
