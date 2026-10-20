precision highp float;

attribute vec2 a_position;
attribute vec2 a_Texcoord;
varying vec2 uv0;
uniform mat4 u_MVP;
void main() 
{ 
    //gl_Position = (vec4(a_position.xy, 0.0, 1.0));
    uv0 = a_Texcoord;
    gl_Position = u_MVP * vec4(a_position.xy, 0.0, 1.0);
    //uv0 = a_Texcoord;
}
