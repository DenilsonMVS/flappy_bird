
#version 460 core

layout (location = 0) in vec2 a_top_left;
layout (location = 1) in vec2 a_top_right;
layout (location = 2) in vec2 a_bot_left;
layout (location = 3) in vec2 a_bot_right;
layout (location = 4) in vec2 a_uv_min;
layout (location = 5) in vec2 a_uv_max;

out vec2 v_tex_coord;

uniform mat4 u_projection;

void main() {
    vec2 positions[4] = vec2[](
        a_top_left,
        a_top_right,
        a_bot_left,
        a_bot_right
    );

    vec2 uvs[4] = vec2[](
        vec2(a_uv_min.x, a_uv_min.y),
        vec2(a_uv_max.x, a_uv_min.y),
        vec2(a_uv_min.x, a_uv_max.y),
        vec2(a_uv_max.x, a_uv_max.y)
    );

    gl_Position = u_projection * vec4(positions[gl_VertexID], 0.0, 1.0);
    v_tex_coord = uvs[gl_VertexID];
}
