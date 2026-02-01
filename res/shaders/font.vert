#version 460 core

layout (location = 0) in ivec2 a_bound_min; 
layout (location = 1) in ivec2 a_bound_max;
layout (location = 2) in vec3 a_color;
layout (location = 3) in uint a_glyph_index;

out vec2 v_tex_coord;
out vec3 v_color;

uniform mat4 u_projection;
uniform sampler2D u_texture;
uniform uint u_glyph_size;
uniform uint u_glyph_margin;

const float WORLD_SCALE = 4.0 / 32767.0;

const vec2 OFFSETS[4] = vec2[](
	vec2(0.0, 0.0),
	vec2(0.0, 1.0),
	vec2(1.0, 0.0),
	vec2(1.0, 1.0)
);

void main() {
	vec2 offset = OFFSETS[gl_VertexID % 4];

	vec2 min_pos = vec2(a_bound_min) * WORLD_SCALE;
    vec2 max_pos = vec2(a_bound_max) * WORLD_SCALE;

	vec2 pos = mix(min_pos, max_pos, offset);
    gl_Position = u_projection * vec4(pos, 0.0, 1.0);

	ivec2 atlas_size = textureSize(u_texture, 0);
	uint glyph_space = u_glyph_size + u_glyph_margin;
	uint columns = uint(atlas_size.x) / glyph_space;

	uint col = a_glyph_index % columns;
	uint row = a_glyph_index / columns;

	vec2 base_pixel = vec2(float(col * glyph_space), float(row * glyph_space));
	vec2 corrected_offset = vec2(offset.x, 1.0 - offset.y);

	v_tex_coord = (base_pixel + corrected_offset * float(u_glyph_size)) / vec2(atlas_size);
	v_color = a_color;
}
