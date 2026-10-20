precision highp float;
precision highp int;

uniform vec4 u_ScreenParams;
uniform mediump int u_skipSample;
uniform float u_rotationFloatVector[6];
uniform vec2 u_positionVec2Vector[6];
uniform vec2 u_pivotVec2Vector[6];
uniform vec2 u_scaleVec2Vector[6];
uniform float u_mirrorEdge;
uniform mediump sampler2D u_inputImageTexture;
uniform float u_minSamples;
uniform float u_maxSamples;
uniform float u_dither;

varying vec2 uv0;

vec2 _f0(inout vec2 _p0, float _p1, vec2 _p2, vec2 _p3, vec2 _p4)
{
    _p0 -= _p2;
    _p0 -= vec2(0.5);
    _p0.y *= (u_ScreenParams.y / u_ScreenParams.x);
    float _66 = sin(_p1);
    float _69 = cos(_p1);
    _p0 = mat2(vec2(_69, _66), vec2(-_66, _69)) * _p0;
    _p0.y *= (u_ScreenParams.x / u_ScreenParams.y);
    _p0 *= (vec2(1.0) / _p4);
    _p0 += vec2(0.5);
    _p0 += _p3;
    return _p0;
}

vec2 _f1(vec2 _p0)
{
    return abs(mod(_p0 - vec2(1.0), vec2(2.0)) - vec2(1.0));
}

float _f2(vec2 _p0)
{
    return ((step(0.0, _p0.x) * step(0.0, _p0.y)) * step(_p0.x, 1.0)) * step(_p0.y, 1.0);
}

float _f4(float _p0, float _p1)
{
    vec2 _218 = fract(vec2(_p0, _p1) * 13.5170001983642578125);
    vec2 _t3 = _218 + vec2(dot(_218, _218.yx + vec2(22.5410003662109375)));
    return fract((_t3.x + _t3.y) * _t3.y);
}

vec2 _f3(float _p0, vec2 _p1, vec2 _p2, vec2 _p3, vec2 _p4, vec2 _p5, vec2 _p6)
{
    return ((((mix(_p1, _p2, vec2(_p0 * 5.0)) * step(_p0, 0.20000000298023223876953125)) + ((mix(_p2, _p3, vec2((_p0 * 5.0) - 1.0)) * (1.0 - step(_p0, 0.20000000298023223876953125))) * step(_p0, 0.4000000059604644775390625))) + ((mix(_p3, _p4, vec2((_p0 * 5.0) - 2.0)) * (1.0 - step(_p0, 0.4000000059604644775390625))) * step(_p0, 0.60000002384185791015625))) + ((mix(_p4, _p5, vec2((_p0 * 5.0) - 3.0)) * (1.0 - step(_p0, 0.60000002384185791015625))) * step(_p0, 0.800000011920928955078125))) + (mix(_p5, _p6, vec2((_p0 * 5.0) - 4.0)) * (1.0 - step(_p0, 0.800000011920928955078125)));
}

void main()
{
    if (u_skipSample == 1)
    {
        vec2 param = uv0;
        float param_1 = (u_rotationFloatVector[0] * 3.141592502593994140625) / 180.0;
        vec2 param_2 = u_positionVec2Vector[0];
        vec2 param_3 = u_pivotVec2Vector[0];
        vec2 param_4 = u_scaleVec2Vector[0];
        vec2 _281 = _f0(param, param_1, param_2, param_3, param_4);
        float _284 = step(u_mirrorEdge, 0.5);
        vec2 param_5 = _281;
        vec2 _294 = (_281 * _284) + (_f1(param_5) * (1.0 - _284));
        vec2 param_6 = _294;
        gl_FragData[0] = texture2D(u_inputImageTexture, _294) * _f2(param_6);
        return;
    }
    vec2 param_7 = uv0;
    float param_8 = (u_rotationFloatVector[0] * 3.141592502593994140625) / 180.0;
    vec2 param_9 = u_positionVec2Vector[0];
    vec2 param_10 = u_pivotVec2Vector[0];
    vec2 param_11 = u_scaleVec2Vector[0];
    vec2 _326 = _f0(param_7, param_8, param_9, param_10, param_11);
    vec2 param_12 = uv0;
    float param_13 = (u_rotationFloatVector[1] * 3.141592502593994140625) / 180.0;
    vec2 param_14 = u_positionVec2Vector[1];
    vec2 param_15 = u_pivotVec2Vector[1];
    vec2 param_16 = u_scaleVec2Vector[1];
    vec2 _344 = _f0(param_12, param_13, param_14, param_15, param_16);
    vec2 param_17 = uv0;
    float param_18 = (u_rotationFloatVector[2] * 3.141592502593994140625) / 180.0;
    vec2 param_19 = u_positionVec2Vector[2];
    vec2 param_20 = u_pivotVec2Vector[2];
    vec2 param_21 = u_scaleVec2Vector[2];
    vec2 _363 = _f0(param_17, param_18, param_19, param_20, param_21);
    vec2 param_22 = uv0;
    float param_23 = (u_rotationFloatVector[3] * 3.141592502593994140625) / 180.0;
    vec2 param_24 = u_positionVec2Vector[3];
    vec2 param_25 = u_pivotVec2Vector[3];
    vec2 param_26 = u_scaleVec2Vector[3];
    vec2 _382 = _f0(param_22, param_23, param_24, param_25, param_26);
    vec2 param_27 = uv0;
    float param_28 = (u_rotationFloatVector[4] * 3.141592502593994140625) / 180.0;
    vec2 param_29 = u_positionVec2Vector[4];
    vec2 param_30 = u_pivotVec2Vector[4];
    vec2 param_31 = u_scaleVec2Vector[4];
    vec2 _401 = _f0(param_27, param_28, param_29, param_30, param_31);
    vec2 param_32 = uv0;
    float param_33 = (u_rotationFloatVector[5] * 3.141592502593994140625) / 180.0;
    vec2 param_34 = u_positionVec2Vector[5];
    vec2 param_35 = u_pivotVec2Vector[5];
    vec2 param_36 = u_scaleVec2Vector[5];
    vec2 _420 = _f0(param_32, param_33, param_34, param_35, param_36);
    float _424 = max(u_minSamples, 2.0);
    float _442 = floor(_424 + ((max(u_maxSamples, _424) - _424) * smoothstep(0.0, 0.20000000298023223876953125, length(_420 - _326))));
    vec4 _t14 = vec4(0.0);
    for (float _t15 = 0.0; _t15 <= 256.0; _t15 += 1.0)
    {
        if (_t15 >= _442)
        {
            break;
        }
        float _t16 = _t15 / (_442 - 1.0);
        float param_37 = _t15 + uv0.x;
        float param_38 = _t15 * uv0.y;
        float _484 = _t16;
        float _485 = _484 + ((u_dither * (_f4(param_37, param_38) - 0.5)) / _442);
        _t16 = _485;
        float param_39 = _485;
        vec2 param_40 = _326;
        vec2 param_41 = _344;
        vec2 param_42 = _363;
        vec2 param_43 = _382;
        vec2 param_44 = _401;
        vec2 param_45 = _420;
        vec2 _501 = _f3(param_39, param_40, param_41, param_42, param_43, param_44, param_45);
        float _503 = step(u_mirrorEdge, 0.5);
        vec2 param_46 = _501;
        vec2 _513 = (_501 * _503) + (_f1(param_46) * (1.0 - _503));
        vec2 param_47 = _513;
        _t14 += (texture2D(u_inputImageTexture, _513) * _f2(param_47));
    }
    vec4 _526 = _t14;
    vec4 _528 = _526 / vec4(_442);
    _t14 = _528;
    gl_FragData[0] = _528;
}

