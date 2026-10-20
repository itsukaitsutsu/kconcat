precision highp float;

attribute vec3 position;
attribute vec4 color;
attribute vec2 texcoord0;
varying highp vec2 uv0;
//uniform mat4 u_MVP;
void main() 
{ 
    //gl_Position = u_MVP * position;
    //gl_Position = sign(vec4(position.xy, 0.0, 1.0));
    gl_Position = vec4(position.xy * 2.0 - 1.0, 0.0, 1.0);
    uv0 = texcoord0;
    uv0 = vec2(texcoord0.x, 1.0 - texcoord0.y);
    // uv0 = vec2(position.x, 1.0 - position.y);
    // uv0 = vec2(position.x, position.y);
}
