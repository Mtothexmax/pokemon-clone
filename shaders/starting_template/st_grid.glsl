// Starting template floor: a lit surface with a world-space grid, 1 m cells and a stronger line every major_every cells.
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

// coverage of lines `width` cells wide on a unit grid, filtered to the pixel; settles to the mean once a cell is under a pixel
float st_grid_lines(vec2 p, float width) {
    vec2 w = max(fwidth(p), vec2(1e-5));
    vec2 d = abs(fract(p + 0.5) - 0.5);
    vec2 line = 1.0 - smoothstep(vec2(width * 0.5) - w, vec2(width * 0.5) + w, d);
    line = mix(line, vec2(width), smoothstep(0.3, 1.0, w));
    return max(line.x, line.y);
}

void main() {
    StGridMaterial mat = StGridMaterial_unpack(v_material_offset);
    vec2 p = v_world_pos.xz / max(mat.cell_size, 1e-3);
    float minor = st_grid_lines(p, mat.line_width);
    float major = st_grid_lines(p / max(mat.major_every, 1.0), mat.line_width * 2.0 / max(mat.major_every, 1.0));
    float line = max(minor * 0.45, major);

    vec3 n = normalize(v_normal);
    HzSurface s = hz_surface_default();
    s.normal = n;
    s.geo_normal = n;
    s.albedo = mix(mat.color.rgb, mat.line_color.rgb, line);
    s.smoothness = mat.smoothness;
#ifdef HZ_RENDERER_GTAO
    s.ambient_occlusion = min(s.ambient_occlusion, hz_ao(gl_FragCoord.xy));
#endif
    hz_output(s, v_world_pos, v_view_depth, HZ_VELOCITY(v_clip_cur, v_clip_prev));
}
@end
