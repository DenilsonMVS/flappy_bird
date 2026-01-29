#version 460 core

in vec2 v_tex_coord;

uniform sampler2D u_texture; 

out vec4 color;

void main() {
    color = texture(u_texture, v_tex_coord);
}
