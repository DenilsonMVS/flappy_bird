
#version 460 core

layout (location = 0) in ivec2 a_top_left;
layout (location = 1) in ivec2 a_top_right;
layout (location = 2) in ivec2 a_bot_left;
layout (location = 3) in ivec2 a_bot_right;
layout (location = 4) in uvec2 a_uv_min;
layout (location = 5) in uvec2 a_uv_max;

out vec2 v_tex_coord;

uniform mat4 u_projection;
uniform float u_world_scale;
uniform sampler2D u_texture; 

void main() {
    ivec2 positions[4] = ivec2[](
        a_top_left,
        a_top_right,
        a_bot_left,
        a_bot_right
    );

    ivec2 uvs[4] = ivec2[](
        ivec2(a_uv_min.x, a_uv_min.y),
        ivec2(a_uv_max.x, a_uv_min.y),
        ivec2(a_uv_min.x, a_uv_max.y),
        ivec2(a_uv_max.x, a_uv_max.y)
    );

    vec2 tex_size = vec2(textureSize(u_texture, 0));

    gl_Position = u_projection * vec4(vec2(positions[gl_VertexID]) * u_world_scale, 0.0, 1.0);
    v_tex_coord = vec2(uvs[gl_VertexID]) / tex_size;
}
