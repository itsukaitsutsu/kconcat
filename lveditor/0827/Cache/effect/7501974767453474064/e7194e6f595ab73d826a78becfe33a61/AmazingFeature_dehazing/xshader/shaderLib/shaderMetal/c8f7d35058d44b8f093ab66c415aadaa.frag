#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float2 u_ScreenParams;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 uv [[user(locn0)]];
};

static inline __attribute__((always_inline))
float3 _f0(thread const float3& _p0)
{
    float4 _51 = mix(float4(_p0.zy, -1.0, 0.666666686534881591796875), float4(_p0.yz, 0.0, -0.3333333432674407958984375), float4(step(_p0.z, _p0.y)));
    float4 _t1 = _51;
    float4 _t2 = mix(float4(_51.xyw, _p0.x), float4(_p0.x, _51.yzx), float4(step(_t1.x, _p0.x)));
    float _86 = _t2.x - fast::min(_t2.w, _t2.y);
    return float3(abs(_t2.z + ((_t2.w - _t2.y) / ((6.0 * _86) + 1.0000000133514319600180897396058e-10))), _86 / (_t2.x + 1.0000000133514319600180897396058e-10), _t2.x);
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> texture_source [[texture(0)]], sampler texture_sourceSmplr [[sampler(0)]])
{
    main0_out out = {};
    float2 _124 = float2(1.0) / buffer.u_ScreenParams;
    float _t7 = 0.0;
    for (int _t8 = -1; _t8 <= 1; _t8++)
    {
        for (int _t9 = -1; _t9 <= 1; _t9++)
        {
            float3 param = texture_source.sample(texture_sourceSmplr, (in.uv + (float2(float(_t8), float(_t9)) * _124))).xyz;
            float3 _t12 = _f0(param);
            float _184 = (_t12.x - 218.0) / 8.0;
            _t7 += ((_t12.y * _t12.z) * ((0.5 * fast::min(_184 * _184, 1.0)) + 0.5));
        }
    }
    float _204 = _t7;
    float _205 = _204 / 9.0;
    _t7 = _205;
    out.o_fragColor = float4(_205, 0.0, 0.0, 1.0);
    return out;
}

