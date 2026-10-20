precision highp float;
precision highp int;

uniform float scaleX;
uniform mediump sampler2D u_InputTexture;

varying vec2 v_uv;

void main()
{
    vec2 _t0 = v_uv - vec2(0.5);
    _t0.x /= scaleX;
    vec2 _27 = _t0;
    vec2 _29 = _27 + vec2(0.5);
    _t0 = _29;
    gl_FragData[0] = texture2D(u_InputTexture, _29);
}

