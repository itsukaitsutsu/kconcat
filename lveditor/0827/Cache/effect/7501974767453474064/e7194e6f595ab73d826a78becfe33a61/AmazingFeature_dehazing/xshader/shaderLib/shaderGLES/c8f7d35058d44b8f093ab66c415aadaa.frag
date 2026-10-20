precision highp float;
precision highp int;

uniform vec2 u_ScreenParams;
uniform mediump sampler2D texture_source;

varying vec2 uv;

vec3 _f0(vec3 _p0)
{
    vec4 _51 = mix(vec4(_p0.zy, -1.0, 0.666666686534881591796875), vec4(_p0.yz, 0.0, -0.3333333432674407958984375), vec4(step(_p0.z, _p0.y)));
    vec4 _t1 = _51;
    vec4 _t2 = mix(vec4(_51.xyw, _p0.x), vec4(_p0.x, _51.yzx), vec4(step(_t1.x, _p0.x)));
    float _86 = _t2.x - min(_t2.w, _t2.y);
    return vec3(abs(_t2.z + ((_t2.w - _t2.y) / ((6.0 * _86) + 1.0000000133514319600180897396058e-10))), _86 / (_t2.x + 1.0000000133514319600180897396058e-10), _t2.x);
}

void main()
{
    vec2 _124 = vec2(1.0) / u_ScreenParams;
    float _t7 = 0.0;
    for (mediump int _t8 = -1; _t8 <= 1; _t8++)
    {
        for (mediump int _t9 = -1; _t9 <= 1; _t9++)
        {
            vec3 param = texture2D(texture_source, uv + (vec2(float(_t8), float(_t9)) * _124)).xyz;
            vec3 _t12 = _f0(param);
            float _184 = (_t12.x - 218.0) / 8.0;
            _t7 += ((_t12.y * _t12.z) * ((0.5 * min(_184 * _184, 1.0)) + 0.5));
        }
    }
    float _204 = _t7;
    float _205 = _204 / 9.0;
    _t7 = _205;
    gl_FragData[0] = vec4(_205, 0.0, 0.0, 1.0);
}

