precision highp float;

attribute vec2 a_position;
attribute vec2 a_Texcoord;
attribute vec4 a_color;
attribute vec4 a_bloomPara;
varying vec2 uv0;
varying vec4 v_color;
varying vec4 v_bloomPara;
uniform mat4 u_MVP;


varying vec2 v_local_uv;
varying vec2 v_screen_uv;
varying vec2 m;
varying vec2 n;

uniform mat4 u_InvModel;
uniform vec4 u_ScreenParams;

void main() 
{
    uv0 = a_Texcoord;
    v_color = a_color;
    v_bloomPara = a_bloomPara;


    vec2 position = a_position;
    v_local_uv = a_Texcoord * 2.0 - 1.0;
    gl_Position = vec4(v_local_uv.xy, 0.0, 1.0);
    float y = (v_local_uv.y / position.y);
    float x = (v_local_uv.x / position.x);
    y = v_local_uv.y - position.y;
    x = v_local_uv.x - position.x;
    v_local_uv.x *= u_ScreenParams.x / u_ScreenParams.y;
    v_local_uv = (u_InvModel * vec4(v_local_uv, 0.0, 1.0)).xy;
    vec2 pos = vec2(position.x, (1. - (position.y * 0.5 + 0.5)) * 2.0 - 1.0);
    m = pos.xy;
    n = pos.xy * (a_Texcoord * 2.0 - 1.0);
    v_screen_uv = a_Texcoord;
}
