precision highp float;

varying vec2 uv;

uniform sampler2D resultTexture;
uniform sampler2D srcTexture;
uniform float intensity;

void main()
{
    // origin input
    vec4 srcCol = texture2D(srcTexture, uv);

    // algorithm result
    vec2 uv1 = vec2(uv.x, 1.0 - uv.y);
    vec4 retCol = texture2D(resultTexture, uv1);

    // blend
    float k = clamp(intensity, 0.0, 1.0);
    gl_FragColor = srcCol * (1.0 - k) + retCol * k;
    // gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}
