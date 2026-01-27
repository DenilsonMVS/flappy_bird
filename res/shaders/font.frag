#version 330 core

in vec2 v_tex_coord;

uniform sampler2D u_texture; 
uniform float u_px_range;

out vec4 color;

const float SHARPNESS = 1.5;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec3 msd = texture(u_texture, v_tex_coord).rgb;
	float sd = median(msd.r, msd.g, msd.b);

	vec2 msdf_unit = u_px_range / vec2(textureSize(u_texture, 0));
	vec2 screen_tex_size = vec2(1.0) / fwidth(v_tex_coord);

	float screen_px_range = max(0.5 * dot(msdf_unit, screen_tex_size), 1.0);
	float screen_px_distance = screen_px_range * (sd - 0.5) * SHARPNESS;
    float opacity = clamp(screen_px_distance + 0.5, 0.0, 1.0);

	color = vec4(1.0, 1.0, 1.0, opacity);
}
