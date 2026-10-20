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

// Implementation of the GLSL radians() function
template<typename T>
inline T radians(T d)
{
    return d * T(0.01745329251);
}

struct buffer_t
{
    float2 textExpandRatio;
    float4 u_ScreenParams;
    int u_layerNum;
    spvUnsafeArray<float2, 8> u_rotation;
    spvUnsafeArray<float2, 8> u_position;
    spvUnsafeArray<float2, 8> u_scale;
    int u_isAssociation;
    float u_oriRotation;
    float2 u_oriPosition;
    float2 u_oriPivot;
    float2 u_oriScale;
    spvUnsafeArray<float4, 8> u_bgColor;
    spvUnsafeArray<float, 8> u_bgPad;
    spvUnsafeArray<float, 8> u_bgRadius;
    spvUnsafeArray<float, 8> u_bgFeather;
    spvUnsafeArray<float4, 8> u_layerColor;
    float4 u_oriBgColor;
    float u_oriBgPad;
    float u_oriBgRadius;
    float u_oriBgFeather;
    float u_oriAlpha;
    float3 u_oriColor;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 v_uv [[user(locn0)]];
};

static inline __attribute__((always_inline))
float2 _f0(thread float2& _p0, thread const float& _p1, thread const float2& _p2, thread const float2& _p3, thread const float2& _p4, constant float2& textExpandRatio, constant float4& u_ScreenParams)
{
    _p0 -= (_p2 / textExpandRatio);
    _p0 -= float2(0.5);
    _p0.y *= (u_ScreenParams.y / u_ScreenParams.x);
    float _64 = radians(_p1);
    float _67 = sin(_64);
    float _70 = cos(_64);
    _p0 = float2x2(float2(_70, _67), float2(-_67, _70)) * _p0;
    _p0.y *= (u_ScreenParams.x / u_ScreenParams.y);
    _p0 *= (float2(1.0) / fast::max(_p4, float2(9.9999997473787516355514526367188e-06)));
    _p0 += float2(0.5);
    _p0 += _p3;
    return _p0;
}

static inline __attribute__((always_inline))
float _f3(thread const float2& _p0, thread const float& _p1, thread const float& _p2, thread const float& _p3)
{
    return 1.0 - smoothstep(0.0, fast::max(_p3, 9.9999999747524270787835121154785e-07), length(fast::max(abs(_p0 - float2(0.5)) - (float2(0.5 + _p1) - float2(_p2)), float2(0.0))) - _p2);
}

static inline __attribute__((always_inline))
float4 _f2(thread const float4& _p0, thread const float4& _p1)
{
    float4 _t4 = _p0;
    float4 _t5 = _p1;
    float3 _142;
    if (_t4.w > 9.9999997473787516355514526367188e-06)
    {
        _142 = _p0.xyz / float3(_t4.w);
    }
    else
    {
        _142 = float3(0.0);
    }
    float3 _158;
    if (_t5.w > 9.9999997473787516355514526367188e-06)
    {
        _158 = _p1.xyz / float3(_t5.w);
    }
    else
    {
        _158 = float3(0.0);
    }
    return float4((((_158 * _t5.w) * (1.0 - _t4.w)) + ((_142 * _t4.w) * (1.0 - _t5.w))) + (_142 * (_t4.w * _t5.w)), _t4.w + (_t5.w * (1.0 - _t4.w)));
}

static inline __attribute__((always_inline))
float _f1(thread const float2& _p0)
{
    return ((step(0.0, _p0.x) * step(0.0, _p0.y)) * step(_p0.x, 1.0)) * step(_p0.y, 1.0);
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTexture [[texture(0)]], sampler u_inputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    float4 _t15 = float4(0.0);
    for (int _t16 = 0; _t16 < buffer.u_layerNum; _t16++)
    {
        float2 _t17 = in.v_uv;
        float2 param = in.v_uv;
        float param_1 = buffer.u_rotation[_t16].x;
        float2 param_2 = buffer.u_position[_t16];
        float2 param_3 = float2(0.0);
        float2 param_4 = buffer.u_scale[_t16];
        float2 _288 = _f0(param, param_1, param_2, param_3, param_4, buffer.textExpandRatio, buffer.u_ScreenParams);
        _t17 = _288;
        if (buffer.u_isAssociation == 1)
        {
            float2 param_5 = _t17;
            float param_6 = buffer.u_oriRotation;
            float2 param_7 = buffer.u_oriPosition;
            float2 param_8 = buffer.u_oriPivot;
            float2 param_9 = buffer.u_oriScale;
            float2 _309 = _f0(param_5, param_6, param_7, param_8, param_9, buffer.textExpandRatio, buffer.u_ScreenParams);
            _t17 = _309;
        }
        float4 _t18 = float4(0.0);
        bool _317 = buffer.u_bgColor[_t16].w > 9.9999997473787516355514526367188e-06;
        bool _327;
        if (_317)
        {
            _327 = buffer.u_bgPad[_t16] > 0.0;
        }
        else
        {
            _327 = _317;
        }
        if (_327)
        {
            float2 param_10 = ((_t17 - float2(0.5)) * buffer.textExpandRatio) + float2(0.5);
            float param_11 = buffer.u_bgPad[_t16];
            float param_12 = buffer.u_bgRadius[_t16];
            float param_13 = buffer.u_bgFeather[_t16];
            float4 param_14 = buffer.u_bgColor[_t16] * _f3(param_10, param_11, param_12, param_13);
            float4 param_15 = _t18;
            _t18 = _f2(param_14, param_15);
        }
        float2 param_16 = _t17;
        float4 param_17 = float4(buffer.u_layerColor[_t16].xyz, 1.0) * ((u_inputTexture.sample(u_inputTextureSmplr, _t17).w * _f1(param_16)) * buffer.u_layerColor[_t16].w);
        float4 param_18 = _t18;
        float4 _403 = _f2(param_17, param_18);
        _t18 = _403;
        float4 param_19 = _t15;
        float4 param_20 = _403;
        _t15 = _f2(param_19, param_20);
    }
    float2 param_21 = in.v_uv;
    float param_22 = buffer.u_oriRotation;
    float2 param_23 = buffer.u_oriPosition;
    float2 param_24 = buffer.u_oriPivot;
    float2 param_25 = buffer.u_oriScale;
    float2 _422 = _f0(param_21, param_22, param_23, param_24, param_25, buffer.textExpandRatio, buffer.u_ScreenParams);
    float4 _t25 = float4(0.0);
    if ((buffer.u_oriBgColor.w > 9.9999997473787516355514526367188e-06) && (buffer.u_oriBgPad > 0.0))
    {
        float2 param_26 = ((_422 - float2(0.5)) * buffer.textExpandRatio) + float2(0.5);
        float param_27 = buffer.u_oriBgPad;
        float param_28 = buffer.u_oriBgRadius;
        float param_29 = buffer.u_oriBgFeather;
        float4 param_30 = buffer.u_oriBgColor * _f3(param_26, param_27, param_28, param_29);
        float4 param_31 = _t25;
        _t25 = _f2(param_30, param_31);
    }
    float2 param_32 = _422;
    float4 _476 = (u_inputTexture.sample(u_inputTextureSmplr, _422) * _f1(param_32)) * buffer.u_oriAlpha;
    float4 _t29 = _476;
    float3 _489 = _476.xyz * float3(buffer.u_oriColor.x, buffer.u_oriColor.y, buffer.u_oriColor.z);
    _t29.x = _489.x;
    _t29.y = _489.y;
    _t29.z = _489.z;
    float4 param_33 = _t29;
    float4 param_34 = _t25;
    float4 _500 = _f2(param_33, param_34);
    _t25 = _500;
    float4 param_35 = _500;
    float4 param_36 = _t15;
    out.o_fragColor = _f2(param_35, param_36);
    return out;
}

