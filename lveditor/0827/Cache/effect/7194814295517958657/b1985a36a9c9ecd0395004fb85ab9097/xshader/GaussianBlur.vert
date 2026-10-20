precision highp float;

attribute vec2 position;
attribute vec2 texcoord0;
attribute vec4 a_bloomPara;

varying vec2 uv0;
varying vec4 v_bloomPara;
//uniform mat4 u_MVP;
void main() 
{ 
    //gl_Position = u_MVP * position;
    gl_Position = sign(vec4(texcoord0*2.0-1.0, 0.0, 1.0));
    uv0 = texcoord0;
    v_bloomPara = a_bloomPara;
}
