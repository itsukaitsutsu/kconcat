precision highp float;

attribute vec2 position;
attribute vec2 texcoord0;
attribute vec4 a_color;
attribute vec4 a_bloomPara;
attribute vec4 a_bloomPara2;

varying vec2 uv0;
varying vec4 v_color;
varying vec4 v_bloomPara;
varying vec4 v_bloomPara2;

void main() 
{ 
    v_color = a_color;
    v_bloomPara = a_bloomPara;
    v_bloomPara2 = a_bloomPara2;
    gl_Position = sign(vec4(texcoord0*2.0-1.0, 0.0, 1.0));
    uv0 = texcoord0;
}
