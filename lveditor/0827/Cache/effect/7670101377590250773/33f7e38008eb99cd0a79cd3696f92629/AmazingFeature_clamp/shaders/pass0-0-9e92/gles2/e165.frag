precision highp float;

uniform sampler2D inputImageTexture;
varying vec2 uv;

void main()
{
    vec4 col = texture2D(inputImageTexture,uv);
    gl_FragColor = vec4(clamp(col,0.,1.));
}
