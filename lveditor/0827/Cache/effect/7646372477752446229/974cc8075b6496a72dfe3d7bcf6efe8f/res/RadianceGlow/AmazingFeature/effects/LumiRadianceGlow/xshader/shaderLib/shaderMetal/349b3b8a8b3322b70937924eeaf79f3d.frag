#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    int u_useMask;
    float u_grayScale;
    int u_thresholdType;
    float u_thresholdLow;
    float u_thresholdHigh;
    float u_thresholdSmooth;
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
float _f0(thread const float3& _p0)
{
    return dot(_p0, float3(0.2125999927520751953125, 0.715200006961822509765625, 0.072200000286102294921875));
}

static inline __attribute__((always_inline))
float _f3(thread float& _p0, thread const float& _p1, thread const float& _p2)
{
    if ((_p0 <= _p1) || (_p0 > _p2))
    {
        _p0 = 0.0;
    }
    else
    {
        _p0 = (_p0 - _p1) / (1.0 - _p1);
    }
    return _p0;
}

static inline __attribute__((always_inline))
float4 _f4(thread float4& _p0, thread const float& _p1, thread const float& _p2)
{
    float param = _p0.x;
    float param_1 = _p1;
    float param_2 = _p2;
    float _154 = _f3(param, param_1, param_2);
    float param_3 = _p0.y;
    float param_4 = _p1;
    float param_5 = _p2;
    float _163 = _f3(param_3, param_4, param_5);
    float param_6 = _p0.z;
    float param_7 = _p1;
    float param_8 = _p2;
    float _172 = _f3(param_6, param_7, param_8);
    float _180 = _p0.x;
    float _182 = _p0.y;
    float _185 = _p0.z;
    float4 _190 = _p0;
    float3 _192 = _190.xyz * (((_154 + _163) + _172) / fast::max((_180 + _182) + _185, 9.9999997473787516355514526367188e-06));
    _p0.x = _192.x;
    _p0.y = _192.y;
    _p0.z = _192.z;
    return _p0;
}

static inline __attribute__((always_inline))
float _f1(thread float& _p0, thread const float& _p1, thread const float& _p2, thread const float& _p3)
{
    if (_p0 <= _p1)
    {
        _p0 = ((_p3 * _p0) * _p0) / fast::max(_p1, 9.9999997473787516355514526367188e-06);
    }
    else
    {
        if (_p0 > _p2)
        {
            _p0 = _p3 * (fma(_p0, _p0, -(_p2 * _p0)) + _p2);
        }
    }
    return _p0;
}

static inline __attribute__((always_inline))
float4 _f2(thread float4& _p0, thread const float& _p1, thread const float& _p2, thread const float& _p3)
{
    float param = _p0.x;
    float param_1 = _p1;
    float param_2 = _p2;
    float param_3 = _p3;
    float _96 = _f1(param, param_1, param_2, param_3);
    _p0.x = _96;
    float param_4 = _p0.y;
    float param_5 = _p1;
    float param_6 = _p2;
    float param_7 = _p3;
    float _108 = _f1(param_4, param_5, param_6, param_7);
    _p0.y = _108;
    float param_8 = _p0.z;
    float param_9 = _p1;
    float param_10 = _p2;
    float param_11 = _p3;
    float _120 = _f1(param_8, param_9, param_10, param_11);
    _p0.z = _120;
    return _p0;
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_maskTex [[texture(0)]], texture2d<float> u_inputTexture [[texture(1)]], sampler u_maskTexSmplr [[sampler(0)]], sampler u_inputTextureSmplr [[sampler(1)]])
{
    main0_out out = {};
    if (buffer.u_useMask == 1)
    {
        float4 _220 = u_maskTex.sample(u_maskTexSmplr, in.uv0);
        float4 _t4 = _220;
        float3 param = _220.xyz;
        float4 _226 = _t4;
        float3 _234 = mix(_226.xyz, float3(_f0(param)), float3(buffer.u_grayScale));
        _t4.x = _234.x;
        _t4.y = _234.y;
        _t4.z = _234.z;
        out.o_fragColor = _t4;
    }
    else
    {
        float4 _t6 = u_inputTexture.sample(u_inputTextureSmplr, in.uv0);
        if (buffer.u_thresholdType == 0)
        {
            float4 param_1 = _t6;
            float param_2 = buffer.u_thresholdLow;
            float param_3 = buffer.u_thresholdHigh;
            float4 _264 = _f4(param_1, param_2, param_3);
            _t6 = _264;
        }
        else
        {
            float4 param_4 = _t6;
            float param_5 = buffer.u_thresholdLow;
            float param_6 = buffer.u_thresholdHigh;
            float param_7 = buffer.u_thresholdSmooth;
            float4 _275 = _f2(param_4, param_5, param_6, param_7);
            _t6 = _275;
            float3 param_8 = _275.xyz;
            float4 _281 = _t6;
            float3 _287 = mix(_281.xyz, float3(_f0(param_8)), float3(buffer.u_grayScale));
            _t6.x = _287.x;
            _t6.y = _287.y;
            _t6.z = _287.z;
        }
        out.o_fragColor = _t6;
    }
    return out;
}

