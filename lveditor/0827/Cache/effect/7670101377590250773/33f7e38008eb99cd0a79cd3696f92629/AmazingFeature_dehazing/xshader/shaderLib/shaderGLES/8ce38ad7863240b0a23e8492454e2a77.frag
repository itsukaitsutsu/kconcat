precision highp float;
precision highp int;

uniform vec2 u_ScreenParams;
uniform vec4 u_offsets;
uniform mediump sampler2D texture_source;
uniform float strength;
uniform mediump sampler2D texture_skin;
uniform mediump sampler2D texture_trans;
uniform float tv;
uniform float human;
uniform vec4 u_sliderInfos;
uniform float alpha_s_full;
uniform mediump sampler2D texture_curves;
uniform mediump sampler2D texture_color;

varying vec2 uv;

float _f2(vec3 _p0, float _p1)
{
    return exp(((-0.5) * dot(_p0, _p0)) / (_p1 * _p1));
}

vec3 _f0(vec3 _p0)
{
    vec4 _59 = mix(vec4(_p0.zy, -1.0, 0.666666686534881591796875), vec4(_p0.yz, 0.0, -0.3333333432674407958984375), vec4(step(_p0.z, _p0.y)));
    vec4 _t1 = _59;
    vec4 _t2 = mix(vec4(_59.xyw, _p0.x), vec4(_p0.x, _59.yzx), vec4(step(_t1.x, _p0.x)));
    float _94 = _t2.x - min(_t2.w, _t2.y);
    return vec3(abs(_t2.z + ((_t2.w - _t2.y) / ((6.0 * _94) + 1.0000000133514319600180897396058e-10))), _94 / (_t2.x + 1.0000000133514319600180897396058e-10), _t2.x);
}

vec3 _f1(vec3 _p0)
{
    return mix(vec3(1.0), clamp(abs((fract(_p0.xxx + vec3(1.0, 0.666666686534881591796875, 0.3333333432674407958984375)) * 6.0) - vec3(3.0)) - vec3(1.0), vec3(0.0), vec3(1.0)), vec3(_p0.y)) * _p0.z;
}

void main()
{
    vec2 _180 = vec2(uv.x, 1.0 - uv.y);
    vec2 _t9 = vec2(1.0 / u_ScreenParams.x, 1.0 / u_ScreenParams.y) * min(u_ScreenParams.x, u_ScreenParams.y);
    vec2 _205 = vec2(u_offsets.w, 0.0);
    mat2 _215 = mat2(vec2(_t9.x, 0.0), vec2(0.0, _t9.y));
    mat2 _224 = mat2(vec2(-_t9.x, 0.0), vec2(0.0, _t9.y));
    mat2 _233 = mat2(vec2(_t9.x, 0.0), vec2(0.0, -_t9.y));
    mat2 _243 = mat2(vec2(-_t9.x, 0.0), vec2(0.0, -_t9.y));
    vec2 _t15[10];
    _t15[0] = uv + (_215 * u_offsets.xy);
    _t15[1] = uv + (_224 * u_offsets.xy);
    _t15[2] = uv + (_215 * u_offsets.zz);
    _t15[3] = uv + (_224 * u_offsets.zz);
    _t15[4] = uv + (_215 * _205);
    _t15[5] = uv + (_224 * _205);
    _t15[6] = uv + (_233 * u_offsets.zz);
    _t15[7] = uv + (_243 * u_offsets.zz);
    _t15[8] = uv + (_233 * u_offsets.xy);
    _t15[9] = uv + (_243 * u_offsets.xy);
    float _t16[10];
    _t16[0] = 0.895183026790618896484375;
    _t16[1] = 0.895183026790618896484375;
    _t16[2] = 0.9726979732513427734375;
    _t16[3] = 0.9726979732513427734375;
    _t16[4] = 0.895183026790618896484375;
    _t16[5] = 0.895183026790618896484375;
    _t16[6] = 0.9726979732513427734375;
    _t16[7] = 0.9726979732513427734375;
    _t16[8] = 0.895183026790618896484375;
    _t16[9] = 0.895183026790618896484375;
    vec4 _t17 = texture2D(texture_source, uv);
    if (strength > 0.0089999996125698089599609375)
    {
        float _t18 = 1.0;
        vec3 _t19 = _t17.xyz;
        for (mediump int _t20 = 0; _t20 < 10; _t20++)
        {
            mediump vec3 _377 = texture2D(texture_source, _t15[_t20]).xyz;
            vec3 param = _t17.xyz - _377;
            float param_1 = 0.0199999995529651641845703125 * strength;
            float _392 = _f2(param, param_1);
            _t18 += (_t16[_t20] * _392);
            _t19 += ((_377 * _t16[_t20]) * _392);
        }
        vec3 _410 = _t19 / vec3(_t18);
        _t17.x = _410.x;
        _t17.y = _410.y;
        _t17.z = _410.z;
    }
    mediump vec4 _421 = texture2D(texture_skin, _180);
    mediump float _422 = _421.x;
    mediump vec4 _427 = texture2D(texture_trans, _180);
    mediump float _428 = _427.x;
    float _453 = mix(clamp(strength * 1.5, 0.0, 1.0), strength * (1.0 - (_422 * 0.87999999523162841796875)), human);
    vec3 param_2 = ((_t17.xyz - vec3(0.800000011920928955078125)) / vec3(mix(1.0, max(1.0 - (_453 * 0.4000000059604644775390625), mix(_428, mix(_428, tv, max(_422, 0.300000011920928955078125)), human)), u_sliderInfos.x))) + vec3(0.800000011920928955078125);
    vec3 _481 = _f0(param_2);
    vec3 _t29 = _481;
    vec3 _t33 = _481;
    mediump vec4 _504 = texture2D(texture_curves, vec2(clamp(_t29.z, 0.0, 1.0), 0.0));
    mediump float _505 = _504.z;
    _t33.z = mix(mix(_t29.z, _505, 0.0500000007450580596923828125 * _453), mix(_t29.z, _505, 0.2599999904632568359375 * _453), _422);
    mediump vec4 _529 = texture2D(texture_curves, vec2(clamp(_t29.y, 0.0, 1.0), 0.0));
    mediump float _530 = _529.y;
    _t33.y = mix(_t29.y, mix(_t29.y, _530, alpha_s_full * _453), _422);
    _t33.y = mix(_t33.y, mix(_t33.y, mix(_t33.y, _530, 0.60000002384185791015625 * _453), texture2D(texture_curves, vec2(clamp(1.0 - texture2D(texture_color, uv).x, 0.0, 1.0), 0.0)).x), human);
    vec3 param_3 = _t33;
    gl_FragData[0] = vec4(_f1(param_3), _t17.w);
}

