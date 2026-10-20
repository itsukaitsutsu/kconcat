precision lowp float;
varying highp vec2 uv0;
uniform sampler2D u_BloomTex1;

void main()
{
    gl_FragColor = texture2D(u_BloomTex1, uv0);
}
