precision highp float;

attribute vec2 a_position;
attribute vec2 a_Texcoord;
attribute vec4 a_color;
attribute vec4 a_bloomPara;
varying vec2 uv0;
varying vec4 v_color;
varying vec4 v_bloomPara;
uniform mat4 u_MVP;
void main() 
{ 
//    gl_Position = (vec4(a_position.xy, 0.0, 1.0));
    gl_Position = (vec4(a_Texcoord * 2.0 - 1.0, 0.0, 1.0));
    uv0 = a_Texcoord;
    v_color = a_color;
    v_bloomPara = a_bloomPara;
//    gl_Position = u_MVP * vec4(a_position.xy, 0.0, 1.0);
    //uv0 = a_Texcoord;
}
