
attribute vec3 position;
varying vec2 uv0;
attribute vec2 texcoord0;

void main()
{
    gl_Position = sign(vec4(position.xy, 0.0, 1.0));
    uv0 = texcoord0;
}

