#version 330 core

layout (location = 0) in vec2 a_pos;
layout (location = 1) in uint a_glyph_index;

out vec2 v_tex_coord;

uniform mat4 u_projection;
uniform sampler2D u_texture; 
uniform uint u_glyph_size;
uniform uint u_glyph_margin;

void main() {
	gl_Position = u_projection * vec4(a_pos, 0.0, 1.0);

	ivec2 atlas_size_int = textureSize(u_texture, 0);
	vec2 atlas_size = vec2(atlas_size_int);

	uint glyph_space = u_glyph_size + u_glyph_margin;
	
	uint columns = uint(atlas_size_int.x) / glyph_space;

	uint col = a_glyph_index % columns;
	uint row = a_glyph_index / columns;

	vec2 base_pixel = vec2(float(col * glyph_space), float(row * glyph_space));

	vec2 uv_offsets[6] = vec2[](
		vec2(0.0, 0.0),
		vec2(0.0, 1.0),
		vec2(1.0, 1.0),
		vec2(0.0, 0.0),
		vec2(1.0, 1.0),
		vec2(1.0, 0.0)
	);

	int index = gl_VertexID % 6;
	vec2 offset = uv_offsets[index];

	v_tex_coord = (base_pixel + offset * float(u_glyph_size)) / atlas_size;
}
