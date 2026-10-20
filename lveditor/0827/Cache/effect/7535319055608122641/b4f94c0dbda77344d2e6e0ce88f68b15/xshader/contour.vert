precision highp float;

uniform vec2 u_size;

attribute vec2 a_texcoord0;

varying vec2 v_xy;
varying vec2 v_uv;

void main()
{ 
    gl_Position = vec4(a_texcoord0 * 2.0 - 1.0, 0.0, 1.0);

    v_uv = a_texcoord0;
    v_xy = a_texcoord0 * u_size;
}