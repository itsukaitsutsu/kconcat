precision highp float;
precision highp int;

uniform float u_exposure;
uniform mediump sampler2D u_tex1;
uniform mediump int u_directionNum;
uniform mediump sampler2D u_tex2;
uniform mediump sampler2D u_tex3;
uniform mediump sampler2D u_tex4;
uniform vec3 u_glowColor;
uniform mediump int u_displayGlow;
uniform mediump sampler2D u_inputTexture;

varying vec2 uv0;

vec4 _f0(vec4 _p0, vec4 _p1)
{
    return (-_p0) * _p1 + (_p0 + _p1);
}

void main()
{
    vec4 _t0 = vec4(0.0);
    if (u_exposure > 9.9999997473787516355514526367188e-06)
    {
        _t0 = texture2D(u_tex1, uv0);
    }
    if (u_directionNum >= 2)
    {
        vec4 param = _t0;
        vec4 param_1 = texture2D(u_tex2, uv0);
        _t0 = _f0(param, param_1);
    }
    if (u_directionNum >= 3)
    {
        vec4 param_2 = _t0;
        vec4 param_3 = texture2D(u_tex3, uv0);
        _t0 = _f0(param_2, param_3);
    }
    if (u_directionNum >= 4)
    {
        vec4 param_4 = _t0;
        vec4 param_5 = texture2D(u_tex4, uv0);
        _t0 = _f0(param_4, param_5);
    }
    vec4 _96 = _t0;
    vec3 _98 = _96.xyz * u_glowColor;
    _t0.x = _98.x;
    _t0.y = _98.y;
    _t0.z = _98.z;
    if (u_displayGlow == 1)
    {
        gl_FragData[0] = _t0;
    }
    else
    {
        vec4 param_6 = _t0;
        vec4 param_7 = texture2D(u_inputTexture, uv0);
        _t0 = _f0(param_6, param_7);
    }
    gl_FragData[0] = clamp(_t0, vec4(0.0), vec4(1.0));
}

