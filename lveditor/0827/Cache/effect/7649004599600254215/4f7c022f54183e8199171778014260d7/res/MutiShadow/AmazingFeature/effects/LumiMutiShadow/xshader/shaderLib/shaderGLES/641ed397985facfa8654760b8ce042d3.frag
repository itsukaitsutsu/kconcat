precision highp float;
precision highp int;

uniform vec2 textExpandRatio;
uniform vec4 u_ScreenParams;
uniform mediump int u_layerNum;
uniform vec2 u_rotation[8];
uniform vec2 u_position[8];
uniform vec2 u_scale[8];
uniform mediump int u_isAssociation;
uniform float u_oriRotation;
uniform vec2 u_oriPosition;
uniform vec2 u_oriPivot;
uniform vec2 u_oriScale;
uniform vec4 u_bgColor[8];
uniform float u_bgPad[8];
uniform float u_bgRadius[8];
uniform float u_bgFeather[8];
uniform mediump sampler2D u_inputTexture;
uniform vec4 u_layerColor[8];
uniform vec4 u_oriBgColor;
uniform float u_oriBgPad;
uniform float u_oriBgRadius;
uniform float u_oriBgFeather;
uniform float u_oriAlpha;
uniform vec3 u_oriColor;

varying vec2 v_uv;

vec2 _f0(inout vec2 _p0, float _p1, vec2 _p2, vec2 _p3, vec2 _p4)
{
    _p0 -= (_p2 / textExpandRatio);
    _p0 -= vec2(0.5);
    _p0.y *= (u_ScreenParams.y / u_ScreenParams.x);
    float _64 = radians(_p1);
    float _67 = sin(_64);
    float _70 = cos(_64);
    _p0 = mat2(vec2(_70, _67), vec2(-_67, _70)) * _p0;
    _p0.y *= (u_ScreenParams.x / u_ScreenParams.y);
    _p0 *= (vec2(1.0) / max(_p4, vec2(9.9999997473787516355514526367188e-06)));
    _p0 += vec2(0.5);
    _p0 += _p3;
    return _p0;
}

float _f3(vec2 _p0, float _p1, float _p2, float _p3)
{
    return 1.0 - smoothstep(0.0, max(_p3, 9.9999999747524270787835121154785e-07), length(max(abs(_p0 - vec2(0.5)) - (vec2(0.5 + _p1) - vec2(_p2)), vec2(0.0))) - _p2);
}

vec4 _f2(vec4 _p0, vec4 _p1)
{
    vec4 _t4 = _p0;
    vec4 _t5 = _p1;
    vec3 _142;
    if (_t4.w > 9.9999997473787516355514526367188e-06)
    {
        _142 = _p0.xyz / vec3(_t4.w);
    }
    else
    {
        _142 = vec3(0.0);
    }
    vec3 _158;
    if (_t5.w > 9.9999997473787516355514526367188e-06)
    {
        _158 = _p1.xyz / vec3(_t5.w);
    }
    else
    {
        _158 = vec3(0.0);
    }
    return vec4((((_158 * _t5.w) * (1.0 - _t4.w)) + ((_142 * _t4.w) * (1.0 - _t5.w))) + (_142 * (_t4.w * _t5.w)), _t4.w + (_t5.w * (1.0 - _t4.w)));
}

float _f1(vec2 _p0)
{
    return ((step(0.0, _p0.x) * step(0.0, _p0.y)) * step(_p0.x, 1.0)) * step(_p0.y, 1.0);
}

void main()
{
    vec4 _t15 = vec4(0.0);
    for (mediump int _t16 = 0; _t16 < u_layerNum; _t16++)
    {
        vec2 _t17 = v_uv;
        vec2 param = v_uv;
        float param_1 = u_rotation[_t16].x;
        vec2 param_2 = u_position[_t16];
        vec2 param_3 = vec2(0.0);
        vec2 param_4 = u_scale[_t16];
        vec2 _288 = _f0(param, param_1, param_2, param_3, param_4);
        _t17 = _288;
        if (u_isAssociation == 1)
        {
            vec2 param_5 = _t17;
            float param_6 = u_oriRotation;
            vec2 param_7 = u_oriPosition;
            vec2 param_8 = u_oriPivot;
            vec2 param_9 = u_oriScale;
            vec2 _309 = _f0(param_5, param_6, param_7, param_8, param_9);
            _t17 = _309;
        }
        vec4 _t18 = vec4(0.0);
        bool _317 = u_bgColor[_t16].w > 9.9999997473787516355514526367188e-06;
        bool _327;
        if (_317)
        {
            _327 = u_bgPad[_t16] > 0.0;
        }
        else
        {
            _327 = _317;
        }
        if (_327)
        {
            vec2 param_10 = ((_t17 - vec2(0.5)) * textExpandRatio) + vec2(0.5);
            float param_11 = u_bgPad[_t16];
            float param_12 = u_bgRadius[_t16];
            float param_13 = u_bgFeather[_t16];
            vec4 param_14 = u_bgColor[_t16] * _f3(param_10, param_11, param_12, param_13);
            vec4 param_15 = _t18;
            _t18 = _f2(param_14, param_15);
        }
        vec2 param_16 = _t17;
        vec4 param_17 = vec4(u_layerColor[_t16].xyz, 1.0) * ((texture2D(u_inputTexture, _t17).w * _f1(param_16)) * u_layerColor[_t16].w);
        vec4 param_18 = _t18;
        vec4 _403 = _f2(param_17, param_18);
        _t18 = _403;
        vec4 param_19 = _t15;
        vec4 param_20 = _403;
        _t15 = _f2(param_19, param_20);
    }
    vec2 param_21 = v_uv;
    float param_22 = u_oriRotation;
    vec2 param_23 = u_oriPosition;
    vec2 param_24 = u_oriPivot;
    vec2 param_25 = u_oriScale;
    vec2 _422 = _f0(param_21, param_22, param_23, param_24, param_25);
    vec4 _t25 = vec4(0.0);
    if ((u_oriBgColor.w > 9.9999997473787516355514526367188e-06) && (u_oriBgPad > 0.0))
    {
        vec2 param_26 = ((_422 - vec2(0.5)) * textExpandRatio) + vec2(0.5);
        float param_27 = u_oriBgPad;
        float param_28 = u_oriBgRadius;
        float param_29 = u_oriBgFeather;
        vec4 param_30 = u_oriBgColor * _f3(param_26, param_27, param_28, param_29);
        vec4 param_31 = _t25;
        _t25 = _f2(param_30, param_31);
    }
    vec2 param_32 = _422;
    vec4 _476 = (texture2D(u_inputTexture, _422) * _f1(param_32)) * u_oriAlpha;
    vec4 _t29 = _476;
    vec3 _489 = _476.xyz * vec3(u_oriColor.x, u_oriColor.y, u_oriColor.z);
    _t29.x = _489.x;
    _t29.y = _489.y;
    _t29.z = _489.z;
    vec4 param_33 = _t29;
    vec4 param_34 = _t25;
    vec4 _500 = _f2(param_33, param_34);
    _t25 = _500;
    vec4 param_35 = _500;
    vec4 param_36 = _t15;
    gl_FragData[0] = _f2(param_35, param_36);
}

