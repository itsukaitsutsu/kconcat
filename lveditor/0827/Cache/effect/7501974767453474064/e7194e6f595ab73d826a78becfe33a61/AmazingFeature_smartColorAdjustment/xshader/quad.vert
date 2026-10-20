precision highp float;

attribute vec3 position;
attribute vec2 texcoord0;

varying vec2 uv;

void main()
{
    gl_Position = vec4(position, 1.0);
    uv = (vec4(texcoord0, 0.0, 1.0)).xy;
}
