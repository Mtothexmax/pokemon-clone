// Brickhollow floor: a LEGO baseplate, one stud per cell_size (line_width = stud radius in cells, line_color = the
// studs' top tint; major_every is unused).
#include "renderer.glsl"

@vs
#define HZ_RENDERER_VERTEX_SHADER
#include "renderer_shaders.glsl"
invariant gl_Position;
out vec3 v_normal;
out vec3 v_world_pos;
out float v_view_depth;
flat out uint v_material_offset;
#ifdef HZ_RENDERER_GBUFFER
out vec4 v_clip_cur;
out vec4 v_clip_prev;
#endif

void main() {
    VertexInstanceData v = unpack_instance_data();
    vec4 world_pos = v.model * vec4(v.pos, 1.0);
    gl_Position = hz_scene.view_proj * world_pos;
    // inverse-transpose for the floor's non-uniform scale: model * n would amplify the quantized normal's x/z by the scale
    mat3 m = mat3(v.model[0].xyz, v.model[1].xyz, v.model[2].xyz);
    v_normal = normalize(m * (v.normal / vec3(dot(m[0], m[0]), dot(m[1], m[1]), dot(m[2], m[2]))));
    v_world_pos = world_pos.xyz;
    v_view_depth = -(hz_scene.view * world_pos).z;
    v_material_offset = v.material_offset;
#ifdef HZ_RENDERER_GBUFFER
    HzVelocity vel = hz_velocity(world_pos, hz_world_pos_prev(v.pos, v));
    v_clip_cur = vel.clip_cur;
    v_clip_prev = vel.clip_prev;
#endif
}
@end

@fs
#undef HZ_RENDERER_VERTEX_SHADER
#define HZ_RENDERER_FRAGMENT_SHADER
#define HZ_RENDERER_LIGHTING_BASIC
#define HZ_RENDERER_OUTPUT
#include "renderer_shaders.glsl"
in vec3 v_normal;
in vec3 v_world_pos;
in float v_view_depth;
flat in uint v_material_offset;
#ifdef HZ_RENDERER_GBUFFER
in vec4 v_clip_cur;
in vec4 v_clip_prev;
#endif

@material
struct StGridMaterial {
    @color
    vec4 color;        // the cells
    @color
    vec4 line_color;   // the lines
    float cell_size;   // m per cell
    float major_every; // cells between major lines
    float line_width;  // a minor line's width in cells; major lines are twice as wide
    float smoothness;  // 1 - roughness
};

// LEGO baseplate: one stud per cell. The stud is shaded with a bent normal (its rim catches the sun, its top is flat)
// and a soft contact shadow around it; once a cell is smaller than a few pixels it fades to the plain plate.
void main() {
    StGridMaterial mat = StGridMaterial_unpack(v_material_offset);
    vec2 p = v_world_pos.xz / max(mat.cell_size, 1e-3);
    vec2 f = fract(p) - 0.5;
    float d = length(f);
    float r = mat.line_width;               // stud radius in cells
    float px = max(fwidth(p.x), fwidth(p.y)); // cells per pixel
    float detail = 1.0 - smoothstep(0.12, 0.35, px);
    float aa = max(px, 1e-4);
    float top = 1.0 - smoothstep(r - 0.06 - aa, r - 0.06 + aa, d);  // the flat top
    float side = (1.0 - smoothstep(r - aa, r + aa, d)) - top;        // the rounded rim
    float shadow = smoothstep(r + 0.16, r, d) * (1.0 - top - side);  // contact shadow on the plate

    vec3 n = normalize(v_normal);
    vec2 dir = d > 1e-4 ? f / d : vec2(0.0);
    vec3 bent = normalize(vec3(dir.x, 0.9, dir.y));
    n = normalize(mix(n, bent, side * detail));

    HzSurface s = hz_surface_default();
    s.normal = n;
    s.geo_normal = normalize(v_normal);
    vec3 albedo = mat.color.rgb;
    albedo = mix(albedo, mat.line_color.rgb, top * detail * 0.6);
    albedo *= 1.0 - shadow * detail * 0.35;
    s.albedo = albedo;
    s.smoothness = mix(mat.smoothness, mat.smoothness + 0.25, (top + side) * detail);
#ifdef HZ_RENDERER_GTAO
    s.ambient_occlusion = min(s.ambient_occlusion, hz_ao(gl_FragCoord.xy));
#endif
    hz_output(s, v_world_pos, v_view_depth, HZ_VELOCITY(v_clip_cur, v_clip_prev));
}
@end
