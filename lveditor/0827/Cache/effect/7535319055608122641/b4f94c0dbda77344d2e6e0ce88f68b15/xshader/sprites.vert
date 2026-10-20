precision highp float;

uniform mat4 u_matMask;
uniform mat4 u_matAnim;

attribute vec4 a_position;
attribute vec2 a_texcoord0;

varying vec2 v_uvText;
varying vec2 v_uvMask;
varying vec2 v_uvAnim;

void main()
{
    gl_Position = vec4(a_texcoord0 * 2.0 - 1.0, 0.0, 1.0);
    v_uvText = vec2(a_texcoord0.x, 1.0 - a_texcoord0.y);
    v_uvMask = (u_matMask * vec4(a_texcoord0, 0.0, 1.0)).st;
    v_uvAnim = (u_matAnim * vec4(a_texcoord0, 0.0, 1.0)).st;
}