
#version 460 core

layout (location = 0) in ivec2 a_top_left;
layout (location = 1) in ivec2 a_top_right;
layout (location = 2) in uvec2 a_uv_min;
layout (location = 3) in uvec2 a_uv_max;

out vec2 v_tex_coord;

uniform mat4 u_projection;
uniform float u_world_scale;
uniform sampler2D u_texture; 

void main() {
    vec2 top_left = vec2(a_top_left);
    vec2 top_right = vec2(a_top_right);

    // Use absolute values for dimensions to ensure that flipping UV coordinates 
    // for mirroring doesn't result in a negative aspect ratio, which would 
    // invert the 'edge_side' vector and break geometry reconstruction.
    vec2 uv_size = abs(vec2(a_uv_max) - vec2(a_uv_min));
    float aspect_ratio = uv_size.y / uv_size.x;

    vec2 edge_top = top_right - top_left;
    vec2 edge_side = vec2(-edge_top.y, edge_top.x) * aspect_ratio;
    
    vec2 pos_bot_left  = top_left - edge_side;
    vec2 pos_bot_right = top_right - edge_side;

    vec2 positions[4] = vec2[](
        top_left,
        top_right,
        pos_bot_left,
        pos_bot_right
    );

    // The UV mapping is assigned based on the order of vertices. 
    // By swapping min/max values in the input buffer, we effectively 
    // mirror the texture sample without changing the quad's winding order.
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
