#version 460
#if VERTEX_SHADER

layout(location = 0) in vec4 i_pos;
layout(location = 1) in vec4 i_color;

layout(location = 0) out vec4 f_color;

void main(){
    f_color = i_color;
    gl_Position = vec4(i_pos.xyz, 1.);
}

#endif
#if FRAGMENT_SHADER

layout(location = 0) in vec4 f_color;

layout(location = 0) out vec4 o_color;

void main(){
    o_color = f_color;
}

#endif
