#version 460 core

in vec2 v_tex_coord;
in vec3 v_color;

uniform sampler2D u_texture; 
uniform float u_px_range;

out vec4 color;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec3 msd = texture(u_texture, v_tex_coord).rgb;
	float sd = median(msd.r, msd.g, msd.b);

	vec2 msdf_unit = u_px_range / vec2(textureSize(u_texture, 0));
	vec2 screen_tex_size = vec2(1.0) / fwidthFine(v_tex_coord);

	float screen_px_range = max(0.5 * dot(msdf_unit, screen_tex_size), 1.0);
	float screen_px_distance = screen_px_range * (sd - 0.5);
    float opacity = smoothstep(-0.5, 0.5, screen_px_distance);

	color = vec4(v_color, opacity);
}
