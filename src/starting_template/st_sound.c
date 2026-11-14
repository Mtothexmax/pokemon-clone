// Sound effects, synthesized at start like the three.js sfx.ts (no files): blips, the golden stud coin, a fanfare,
// an alert, a quack, a kick, a splash of water and one piano tone that is pitched per key. Voice lines and the
// fish splash are files (assets/brickhollow/voices, music). Included by starting_template.c above app_init.

typedef enum SoundId {
  SND_BLIP,
  SND_COIN,
  SND_FANFARE,
  SND_ALERT,
  SND_QUACK,
  SND_KICK,
  SND_SPLASH,
  SND_PIANO, // C4; other notes are pitch = 2^(semitones / 12)
  SND_POP,
  SND_RAIN, // a 2 s loop of soft noise
  SND_COUNT
} SoundId;

#define SND_RATE 44100

typedef struct SoundBank {
  AudioSystem *audio;
  Allocator alloc;
  HzHandleT(AudioSource) clip[SND_COUNT];
} SoundBank;

hz_internal f32 snd_env(f32 t, f32 attack, f32 decay) {
  if (t < attack)
    return t / attack;
  return m_expf(-(t - attack) / decay);
}

// a decaying tone at `freq` from `start` seconds, added into out
hz_internal void snd_tone(f32 *out, u32 n, f32 start, f32 dur, f32 freq, f32 vol, b32 square) {
  u32 a = (u32)(start * SND_RATE), b = MIN(n, (u32)((start + dur) * SND_RATE));
  for (u32 i = a; i < b; i++) {
    f32 t = (f32)(i - a) / SND_RATE;
    f32 ph = t * freq;
    f32 s = square ? (ph - m_floorf(ph) < 0.5f ? 1.0f : -1.0f) * 0.5f : m_sinf(ph * 6.2831853f);
    out[i] += s * vol * snd_env(t, 0.008f, dur * 0.35f);
  }
}

hz_internal u32 snd_noise(u32 *s) {
  *s = *s * 1664525u + 1013904223u;
  return *s >> 8;
}

hz_internal void snd_render(SoundId id, f32 *out, u32 n) {
  u32 seed = 77u;
  switch (id) {
  case SND_BLIP: snd_tone(out, n, 0.0f, 0.09f, 880.0f, 0.25f, true); break;
  case SND_COIN:
    snd_tone(out, n, 0.0f, 0.12f, 987.8f, 0.3f, true);
    snd_tone(out, n, 0.08f, 0.45f, 1318.5f, 0.3f, true);
    break;
  case SND_FANFARE: {
    f32 notes[4] = {523.3f, 659.3f, 784.0f, 1046.5f};
    for (i32 i = 0; i < 4; i++)
      snd_tone(out, n, (f32)i * 0.11f, i == 3 ? 0.7f : 0.2f, notes[i], 0.22f, true);
  } break;
  case SND_ALERT:
    snd_tone(out, n, 0.0f, 0.12f, 1200.0f, 0.25f, true);
    snd_tone(out, n, 0.13f, 0.2f, 1600.0f, 0.25f, true);
    break;
  case SND_QUACK:
    // a nasal sweep down with a little noise
    for (u32 i = 0; i < n && i < (u32)(0.22f * SND_RATE); i++) {
      f32 t = (f32)i / SND_RATE;
      f32 f = 620.0f - t * 1300.0f;
      f32 ph = t * f;
      f32 s = (ph - m_floorf(ph) < 0.3f ? 1.0f : -0.45f) * 0.35f;
      s += ((f32)(snd_noise(&seed) & 0xffff) / 65535.0f - 0.5f) * 0.08f;
      out[i] += s * snd_env(t, 0.01f, 0.08f);
    }
    break;
  case SND_KICK:
    for (u32 i = 0; i < n && i < (u32)(0.2f * SND_RATE); i++) {
      f32 t = (f32)i / SND_RATE;
      out[i] += m_sinf(t * (140.0f - t * 300.0f) * 6.2831853f) * 0.6f * snd_env(t, 0.002f, 0.05f);
    }
    break;
  case SND_SPLASH:
    for (u32 i = 0; i < n && i < (u32)(0.5f * SND_RATE); i++) {
      f32 t = (f32)i / SND_RATE;
      f32 w = ((f32)(snd_noise(&seed) & 0xffff) / 65535.0f - 0.5f);
      out[i] += w * 0.5f * snd_env(t, 0.01f, 0.12f);
    }
    break;
  case SND_PIANO:
    // a bright decaying tone with a few harmonics
    for (u32 i = 0; i < n; i++) {
      f32 t = (f32)i / SND_RATE;
      f32 ph = t * 261.63f * 6.2831853f;
      f32 s = m_sinf(ph) + 0.45f * m_sinf(ph * 2.0f) + 0.2f * m_sinf(ph * 3.0f) + 0.08f * m_sinf(ph * 4.0f);
      out[i] += s * 0.22f * snd_env(t, 0.004f, 0.55f);
    }
    break;
  case SND_POP: snd_tone(out, n, 0.0f, 0.07f, 520.0f, 0.3f, false); break;
  case SND_RAIN: {
    f32 lp = 0.0f;
    for (u32 i = 0; i < n; i++) { // low-passed noise with a few crackles; the ends fade so the loop meets
      f32 wn = ((f32)(snd_noise(&seed) & 0xffff) / 65535.0f - 0.5f);
      lp += (wn - lp) * 0.18f;
      f32 crackle = (snd_noise(&seed) & 0x3ff) == 0 ? 0.6f : 0.0f;
      f32 edge = m_minf(1.0f, m_minf((f32)i, (f32)(n - i)) / 400.0f);
      out[i] += (lp * 0.9f + crackle * wn) * edge;
    }
  } break;
  default: break;
  }
}

hz_internal f32 snd_seconds(SoundId id) {
  switch (id) {
  case SND_FANFARE: return 1.1f;
  case SND_PIANO: return 2.0f;
  case SND_COIN: return 0.6f;
  case SND_SPLASH: return 0.6f;
  case SND_RAIN: return 2.0f;
  default: return 0.4f;
  }
}

// main lane, once
hz_internal void sound_init(SoundBank *b, AudioSystem *audio, Allocator *alloc) {
  b->audio = audio;
  b->alloc = *alloc;
  for (u32 id = 0; id < SND_COUNT; id++) {
    u32 n = (u32)(snd_seconds((SoundId)id) * SND_RATE);
    tctx_temp_allocator_scope(ta, NULL) {
      f32 *buf = ALLOC_ARRAY(&ta.allocator, f32, n);
      i16 *pcm = ALLOC_ARRAY(&ta.allocator, i16, n);
      for (u32 i = 0; i < n; i++)
        buf[i] = 0.0f;
      snd_render((SoundId)id, buf, n);
      for (u32 i = 0; i < n; i++)
        pcm[i] = (i16)(m_clampf(buf[i], -1.0f, 1.0f) * 32000.0f);
      WavFile wav = create_wav_from_samples(pcm, n, SND_RATE);
      b->clip[id] = audio_source_from_wav(audio, &wav, &b->alloc);
    }
  }
}

hz_internal void sound_play(SoundBank *b, SoundId id, f32 pitch, f32 volume) {
  audio_play(b->audio, b->clip[id], &(AudioPlayDesc){.volume = volume, .pitch = pitch, .bus = AUDIO_BUS_SFX});
}
