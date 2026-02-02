#version 460 core

layout (location = 0) in ivec2 a_center;
layout (location = 1) in ivec2 a_top_edge; // Vector: TopRight - TopLeft
layout (location = 2) in uvec2 a_uv_min;
layout (location = 3) in uvec2 a_uv_max;

out vec2 v_tex_coord;

uniform mat4 u_projection;
uniform float u_world_scale;
uniform sampler2D u_texture; 

void main() {
    vec2 center = vec2(a_center);
    vec2 top_edge = vec2(a_top_edge);

    // 1. Calculate aspect ratio from UVs (always positive for geometry)
    vec2 uv_size = abs(vec2(a_uv_max) - vec2(a_uv_min));
    float aspect_ratio = uv_size.y / uv_size.x;

    // 2. Derive the side edge (height) from the top edge
    // Rotating top_edge (x, y) by 90 degrees gives (-y, x)
    // We multiply by aspect_ratio to get the correct height relative to width
    vec2 side_edge = vec2(-top_edge.y, top_edge.x) * aspect_ratio;

    // 3. Reconstruct corners using simple vector addition/subtraction
    // Half-vectors reach from the center to the edges
    vec2 h_top = top_edge * 0.5;
    vec2 h_side = side_edge * 0.5;

    // Positions relative to the center
    vec2 positions[4] = vec2[](
        center - h_top + h_side, // Top-Left
        center + h_top + h_side, // Top-Right
        center - h_top - h_side, // Bot-Left
        center + h_top - h_side  // Bot-Right
    );

    ivec2 uvs[4] = ivec2[](
        ivec2(a_uv_min.x, a_uv_min.y),
        ivec2(a_uv_max.x, a_uv_min.y),
        ivec2(a_uv_min.x, a_uv_max.y),
        ivec2(a_uv_max.x, a_uv_max.y)
    );

    vec2 tex_size = vec2(textureSize(u_texture, 0));
    gl_Position = u_projection * vec4(positions[gl_VertexID] * u_world_scale, 0.0, 1.0);
    v_tex_coord = vec2(uvs[gl_VertexID]) / tex_size;
}
