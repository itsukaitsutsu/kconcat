
attribute vec3 attPosition;
varying vec2 uv;
attribute vec2 attUV;

void main()
{
    gl_Position = vec4(attPosition, 1.0);
    uv = attUV;
}

