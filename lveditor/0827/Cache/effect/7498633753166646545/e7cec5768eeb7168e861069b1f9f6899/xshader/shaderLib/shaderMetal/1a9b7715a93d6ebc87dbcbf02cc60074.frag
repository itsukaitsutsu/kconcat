#pragma clang diagnostic ignored "-Wmissing-prototypes"
#pragma clang diagnostic ignored "-Wmissing-braces"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

template<typename T, size_t Num>
struct spvUnsafeArray
{
    T elements[Num ? Num : 1];
    
    thread T& operator [] (size_t pos) thread
    {
        return elements[pos];
    }
    constexpr const thread T& operator [] (size_t pos) const thread
    {
        return elements[pos];
    }
    
    device T& operator [] (size_t pos) device
    {
        return elements[pos];
    }
    constexpr const device T& operator [] (size_t pos) const device
    {
        return elements[pos];
    }
    
    constexpr const constant T& operator [] (size_t pos) const constant
    {
        return elements[pos];
    }
    
    threadgroup T& operator [] (size_t pos) threadgroup
    {
        return elements[pos];
    }
    constexpr const threadgroup T& operator [] (size_t pos) const threadgroup
    {
        return elements[pos];
    }
};

// Implementation of the GLSL mod() function, which is slightly different than Metal fmod()
template<typename Tx, typename Ty>
inline Tx mod(Tx x, Ty y)
{
    return x - y * floor(x / y);
}

struct buffer_t
{
    float4 u_ScreenParams;
    int u_skipSample;
    spvUnsafeArray<float, 6> u_rotationFloatVector;
    spvUnsafeArray<float2, 6> u_positionVec2Vector;
    spvUnsafeArray<float2, 6> u_pivotVec2Vector;
    spvUnsafeArray<float2, 6> u_scaleVec2Vector;
    float u_mirrorEdge;
    float u_minSamples;
    float u_maxSamples;
    float u_dither;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 uv0 [[user(locn0)]];
};

static inline __attribute__((always_inline))
float2 _f0(thread float2& _p0, thread const float& _p1, thread const float2& _p2, thread const float2& _p3, thread const float2& _p4, constant float4& u_ScreenParams)
{
    _p0 -= _p2;
    _p0 -= float2(0.5);
    _p0.y *= (u_ScreenParams.y / u_ScreenParams.x);
    float _66 = sin(_p1);
    float _69 = cos(_p1);
    _p0 = float2x2(float2(_69, _66), float2(-_66, _69)) * _p0;
    _p0.y *= (u_ScreenParams.x / u_ScreenParams.y);
    _p0 *= (float2(1.0) / _p4);
    _p0 += float2(0.5);
    _p0 += _p3;
    return _p0;
}

static inline __attribute__((always_inline))
float2 _f1(thread const float2& _p0)
{
    return abs(mod(_p0 - float2(1.0), float2(2.0)) - float2(1.0));
}

static inline __attribute__((always_inline))
float _f2(thread const float2& _p0)
{
    return ((step(0.0, _p0.x) * step(0.0, _p0.y)) * step(_p0.x, 1.0)) * step(_p0.y, 1.0);
}

static inline __attribute__((always_inline))
float _f4(thread const float& _p0, thread const float& _p1)
{
    float2 _218 = fract(float2(_p0, _p1) * 13.5170001983642578125);
    float2 _t3 = _218 + float2(dot(_218, _218.yx + float2(22.5410003662109375)));
    return fract((_t3.x + _t3.y) * _t3.y);
}

static inline __attribute__((always_inline))
float2 _f3(thread const float& _p0, thread const float2& _p1, thread const float2& _p2, thread const float2& _p3, thread const float2& _p4, thread const float2& _p5, thread const float2& _p6)
{
    return ((((mix(_p1, _p2, float2(_p0 * 5.0)) * step(_p0, 0.20000000298023223876953125)) + ((mix(_p2, _p3, float2((_p0 * 5.0) - 1.0)) * (1.0 - step(_p0, 0.20000000298023223876953125))) * step(_p0, 0.4000000059604644775390625))) + ((mix(_p3, _p4, float2((_p0 * 5.0) - 2.0)) * (1.0 - step(_p0, 0.4000000059604644775390625))) * step(_p0, 0.60000002384185791015625))) + ((mix(_p4, _p5, float2((_p0 * 5.0) - 3.0)) * (1.0 - step(_p0, 0.60000002384185791015625))) * step(_p0, 0.800000011920928955078125))) + (mix(_p5, _p6, float2((_p0 * 5.0) - 4.0)) * (1.0 - step(_p0, 0.800000011920928955078125)));
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputImageTexture [[texture(0)]], sampler u_inputImageTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    if (buffer.u_skipSample == 1)
    {
        float2 param = in.uv0;
        float param_1 = (buffer.u_rotationFloatVector[0] * 3.141592502593994140625) / 180.0;
        float2 param_2 = buffer.u_positionVec2Vector[0];
        float2 param_3 = buffer.u_pivotVec2Vector[0];
        float2 param_4 = buffer.u_scaleVec2Vector[0];
        float2 _281 = _f0(param, param_1, param_2, param_3, param_4, buffer.u_ScreenParams);
        float _284 = step(buffer.u_mirrorEdge, 0.5);
        float2 param_5 = _281;
        float2 _294 = (_281 * _284) + (_f1(param_5) * (1.0 - _284));
        float2 param_6 = _294;
        out.o_fragColor = u_inputImageTexture.sample(u_inputImageTextureSmplr, _294) * _f2(param_6);
        return out;
    }
    float2 param_7 = in.uv0;
    float param_8 = (buffer.u_rotationFloatVector[0] * 3.141592502593994140625) / 180.0;
    float2 param_9 = buffer.u_positionVec2Vector[0];
    float2 param_10 = buffer.u_pivotVec2Vector[0];
    float2 param_11 = buffer.u_scaleVec2Vector[0];
    float2 _326 = _f0(param_7, param_8, param_9, param_10, param_11, buffer.u_ScreenParams);
    float2 param_12 = in.uv0;
    float param_13 = (buffer.u_rotationFloatVector[1] * 3.141592502593994140625) / 180.0;
    float2 param_14 = buffer.u_positionVec2Vector[1];
    float2 param_15 = buffer.u_pivotVec2Vector[1];
    float2 param_16 = buffer.u_scaleVec2Vector[1];
    float2 _344 = _f0(param_12, param_13, param_14, param_15, param_16, buffer.u_ScreenParams);
    float2 param_17 = in.uv0;
    float param_18 = (buffer.u_rotationFloatVector[2] * 3.141592502593994140625) / 180.0;
    float2 param_19 = buffer.u_positionVec2Vector[2];
    float2 param_20 = buffer.u_pivotVec2Vector[2];
    float2 param_21 = buffer.u_scaleVec2Vector[2];
    float2 _363 = _f0(param_17, param_18, param_19, param_20, param_21, buffer.u_ScreenParams);
    float2 param_22 = in.uv0;
    float param_23 = (buffer.u_rotationFloatVector[3] * 3.141592502593994140625) / 180.0;
    float2 param_24 = buffer.u_positionVec2Vector[3];
    float2 param_25 = buffer.u_pivotVec2Vector[3];
    float2 param_26 = buffer.u_scaleVec2Vector[3];
    float2 _382 = _f0(param_22, param_23, param_24, param_25, param_26, buffer.u_ScreenParams);
    float2 param_27 = in.uv0;
    float param_28 = (buffer.u_rotationFloatVector[4] * 3.141592502593994140625) / 180.0;
    float2 param_29 = buffer.u_positionVec2Vector[4];
    float2 param_30 = buffer.u_pivotVec2Vector[4];
    float2 param_31 = buffer.u_scaleVec2Vector[4];
    float2 _401 = _f0(param_27, param_28, param_29, param_30, param_31, buffer.u_ScreenParams);
    float2 param_32 = in.uv0;
    float param_33 = (buffer.u_rotationFloatVector[5] * 3.141592502593994140625) / 180.0;
    float2 param_34 = buffer.u_positionVec2Vector[5];
    float2 param_35 = buffer.u_pivotVec2Vector[5];
    float2 param_36 = buffer.u_scaleVec2Vector[5];
    float2 _420 = _f0(param_32, param_33, param_34, param_35, param_36, buffer.u_ScreenParams);
    float _424 = fast::max(buffer.u_minSamples, 2.0);
    float _442 = floor(_424 + ((fast::max(buffer.u_maxSamples, _424) - _424) * smoothstep(0.0, 0.20000000298023223876953125, length(_420 - _326))));
    float4 _t14 = float4(0.0);
    for (float _t15 = 0.0; _t15 <= 256.0; _t15 += 1.0)
    {
        if (_t15 >= _442)
        {
            break;
        }
        float _t16 = _t15 / (_442 - 1.0);
        float param_37 = _t15 + in.uv0.x;
        float param_38 = _t15 * in.uv0.y;
        float _484 = _t16;
        float _485 = _484 + ((buffer.u_dither * (_f4(param_37, param_38) - 0.5)) / _442);
        _t16 = _485;
        float param_39 = _485;
        float2 param_40 = _326;
        float2 param_41 = _344;
        float2 param_42 = _363;
        float2 param_43 = _382;
        float2 param_44 = _401;
        float2 param_45 = _420;
        float2 _501 = _f3(param_39, param_40, param_41, param_42, param_43, param_44, param_45);
        float _503 = step(buffer.u_mirrorEdge, 0.5);
        float2 param_46 = _501;
        float2 _513 = (_501 * _503) + (_f1(param_46) * (1.0 - _503));
        float2 param_47 = _513;
        _t14 += (u_inputImageTexture.sample(u_inputImageTextureSmplr, _513) * _f2(param_47));
    }
    float4 _526 = _t14;
    float4 _528 = _526 / float4(_442);
    _t14 = _528;
    out.o_fragColor = _528;
    return out;
}

