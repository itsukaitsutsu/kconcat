#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float u_radiusY;
    float u_sigmaY;
    float u_dy;
    float u_threshold;
    float u_intensity;
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
float _f0(thread const float& _p0, thread const float& _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTexture [[texture(0)]], texture2d<float> u_albedo [[texture(1)]], texture2d<float> mask123 [[texture(2)]], sampler u_inputTextureSmplr [[sampler(0)]], sampler u_albedoSmplr [[sampler(1)]], sampler mask123Smplr [[sampler(2)]])
{
    main0_out out = {};
    float4 _t2 = u_inputTexture.sample(u_inputTextureSmplr, in.uv0);
    if (buffer.u_radiusY < 0.001000000047497451305389404296875)
    {
        out.o_fragColor = _t2;
        return out;
    }
    float _t4 = 0.0;
    float4 _t5 = float4(0.0);
    float _t6 = -10.0;
    while (_t6 <= 10.0)
    {
        if (_t6 > (buffer.u_radiusY + 0.001000000047497451305389404296875))
        {
            break;
        }
        float _83 = -buffer.u_radiusY;
        if (_t6 < _83)
        {
            _t6 = _83;
        }
        float2 _94 = in.uv0 + float2(0.0, _t6);
        float2 _t7 = _94;
        bool _99 = _t7.y >= 0.0;
        bool _106;
        if (_99)
        {
            _106 = _t7.y <= 1.0;
        }
        else
        {
            _106 = _99;
        }
        if (_106)
        {
            float param = _t6;
            float param_1 = buffer.u_sigmaY;
            float _115 = _f0(param, param_1);
            _t4 += _115;
            _t5 += (u_albedo.sample(u_albedoSmplr, _94) * _115);
        }
        _t6 += buffer.u_dy;
    }
    float4 _135 = _t2;
    float4 _137 = _135 - (_t5 / float4(_t4));
    float3 _157 = smoothstep(float4(buffer.u_threshold / 5.0), float4(buffer.u_threshold), abs(_137)).xyz;
    float3 _160 = _137.xyz;
    float3 _162 = _135.xyz + ((_157 * buffer.u_intensity) * _160);
    _t2.x = _162.x;
    _t2.y = _162.y;
    _t2.z = _162.z;
    float4 _t12 = mask123.sample(mask123Smplr, in.uv0);
    float4 _177 = _t2;
    float4 _t13 = _177;
    float3 _187 = _177.xyz + ((_157 * 0.20000000298023223876953125) * _160);
    _t13.x = _187.x;
    _t13.y = _187.y;
    _t13.z = _187.z;
    float4 _199 = mix(_177, _t13, float4(_t12.y));
    _t2 = _199;
    out.o_fragColor = fast::clamp(_199, float4(0.0), float4(1.0));
    return out;
}

